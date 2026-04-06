use anyhow::{anyhow, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;

const MAX_RETRIES: u32 = 3;
const WRITE_BUF_SIZE: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HlsData {
    pub segment_urls: Vec<String>,
    pub init_url: Option<String>,
    pub bandwidth: u64,
    pub resolution: Option<String>,
}

pub fn is_hls_url(url: &str) -> bool {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    path.to_lowercase().ends_with(".m3u8")
}

fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return relative.to_string();
    }

    if let Ok(base_url) = url::Url::parse(base) {
        if let Ok(resolved) = base_url.join(relative) {
            return resolved.to_string();
        }
    }

    let base_dir = base.rsplit_once('/').map(|(b, _)| b).unwrap_or(base);
    format!("{}/{}", base_dir, relative)
}

pub async fn resolve_hls(
    client: &reqwest::Client,
    url: &str,
    headers: &HashMap<String, String>,
) -> Result<(String, HlsData)> {
    let bytes = fetch_playlist(client, url, headers).await?;

    match m3u8_rs::parse_playlist(&bytes) {
        Ok((_, m3u8_rs::Playlist::MasterPlaylist(master))) => {
            let variant = master
                .variants
                .iter()
                .filter(|v| !v.is_i_frame)
                .max_by_key(|v| v.bandwidth)
                .ok_or_else(|| anyhow!("No suitable variant stream found"))?;

            let variant_url = resolve_url(url, &variant.uri);
            let bandwidth = variant.bandwidth;
            let resolution = variant
                .resolution
                .as_ref()
                .map(|r| format!("{}x{}", r.width, r.height));

            let variant_bytes = fetch_playlist(client, &variant_url, headers).await?;
            let media_pl = m3u8_rs::parse_media_playlist_res(&variant_bytes)
                .map_err(|e| anyhow!("Failed to parse variant playlist: {}", e))?;

            let (segment_urls, init_url) = extract_segments(&media_pl, &variant_url);
            let filename = derive_filename(url, init_url.is_some());

            Ok((
                filename,
                HlsData {
                    segment_urls,
                    init_url,
                    bandwidth,
                    resolution,
                },
            ))
        }
        Ok((_, m3u8_rs::Playlist::MediaPlaylist(media_pl))) => {
            let (segment_urls, init_url) = extract_segments(&media_pl, url);
            let filename = derive_filename(url, init_url.is_some());

            Ok((
                filename,
                HlsData {
                    segment_urls,
                    init_url,
                    bandwidth: 0,
                    resolution: None,
                },
            ))
        }
        Err(e) => Err(anyhow!("Failed to parse m3u8 playlist: {}", e)),
    }
}

async fn fetch_playlist(
    client: &reqwest::Client,
    url: &str,
    headers: &HashMap<String, String>,
) -> Result<Vec<u8>> {
    let mut req = client.get(url);
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }
    let response = req.send().await?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "HTTP error fetching playlist: {}",
            response.status()
        ));
    }
    Ok(response.bytes().await?.to_vec())
}

fn extract_segments(
    playlist: &m3u8_rs::MediaPlaylist,
    base_url: &str,
) -> (Vec<String>, Option<String>) {
    let mut init_url = None;
    let mut segment_urls = Vec::new();

    for seg in &playlist.segments {
        if let Some(ref map) = seg.map {
            if init_url.is_none() {
                init_url = Some(resolve_url(base_url, &map.uri));
            }
        }
        if !seg.uri.is_empty() {
            segment_urls.push(resolve_url(base_url, &seg.uri));
        }
    }

    (segment_urls, init_url)
}

fn derive_filename(url: &str, is_fmp4: bool) -> String {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);

    // Walk up path segments to find a meaningful name
    let segments: Vec<&str> = path.split('/').collect();
    let mut name = None;
    for &seg in segments.iter().rev() {
        let base = seg.split('.').next().unwrap_or("");
        if !base.is_empty()
            && base != "manifest"
            && base != "index"
            && base != "master"
            && base != "playlist"
        {
            name = Some(base.to_string());
            break;
        }
    }

    let clean = name.unwrap_or_else(|| "video".to_string());
    let ext = if is_fmp4 { "mp4" } else { "ts" };
    format!("{}.{}", clean, ext)
}

#[allow(clippy::too_many_arguments)]
pub async fn download_hls(
    client: reqwest::Client,
    data: HlsData,
    save_path: PathBuf,
    downloaded: Arc<AtomicU64>,
    pause_flag: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
    speed_limit: u64,
    extra_headers: HashMap<String, String>,
) -> Result<()> {
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&save_path)
        .await?;

    let mut all_urls = Vec::new();
    if let Some(ref init_url) = data.init_url {
        all_urls.push(init_url.clone());
    }
    all_urls.extend(data.segment_urls.iter().cloned());

    for seg_url in &all_urls {
        if cancel_flag.load(Ordering::Relaxed) || pause_flag.load(Ordering::Relaxed) {
            file.flush().await?;
            return Ok(());
        }

        stream_segment_to_file(
            &client,
            seg_url,
            &extra_headers,
            &mut file,
            &downloaded,
            &pause_flag,
            &cancel_flag,
            speed_limit,
        )
        .await?;
    }

    file.flush().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn stream_segment_to_file(
    client: &reqwest::Client,
    url: &str,
    headers: &HashMap<String, String>,
    file: &mut tokio::fs::File,
    downloaded: &AtomicU64,
    pause_flag: &AtomicBool,
    cancel_flag: &AtomicBool,
    speed_limit: u64,
) -> Result<()> {
    for attempt in 0..=MAX_RETRIES {
        if cancel_flag.load(Ordering::Relaxed) || pause_flag.load(Ordering::Relaxed) {
            return Ok(());
        }

        let mut req = client.get(url);
        for (k, v) in headers {
            req = req.header(k.as_str(), v.as_str());
        }

        let response = match req.send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                if attempt == MAX_RETRIES {
                    return Err(anyhow!("HTTP error: {}", r.status()));
                }
                tokio::time::sleep(Duration::from_secs(3u64.pow(attempt))).await;
                continue;
            }
            Err(e) => {
                if attempt == MAX_RETRIES {
                    return Err(e.into());
                }
                tokio::time::sleep(Duration::from_secs(3u64.pow(attempt))).await;
                continue;
            }
        };

        let mut stream = response.bytes_stream();
        let mut buf = Vec::with_capacity(WRITE_BUF_SIZE);
        let mut window_start = Instant::now();
        let mut window_bytes: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            if cancel_flag.load(Ordering::Relaxed) || pause_flag.load(Ordering::Relaxed) {
                if !buf.is_empty() {
                    file.write_all(&buf).await?;
                    downloaded.fetch_add(buf.len() as u64, Ordering::Relaxed);
                }
                return Ok(());
            }

            let chunk = chunk_result?;
            let chunk_len = chunk.len() as u64;
            buf.extend_from_slice(&chunk);

            if speed_limit > 0 {
                window_bytes += chunk_len;
                let elapsed = window_start.elapsed().as_secs_f64();
                let expected = window_bytes as f64 / speed_limit as f64;
                if expected > elapsed + 0.005 {
                    let sleep_ms = ((expected - elapsed) * 1000.0) as u64;
                    tokio::time::sleep(Duration::from_millis(sleep_ms.min(50))).await;
                }
                if window_start.elapsed().as_millis() >= 1000 {
                    window_start = Instant::now();
                    window_bytes = 0;
                }
            }

            if buf.len() >= WRITE_BUF_SIZE {
                file.write_all(&buf).await?;
                downloaded.fetch_add(buf.len() as u64, Ordering::Relaxed);
                buf.clear();
            }
        }

        if !buf.is_empty() {
            file.write_all(&buf).await?;
            downloaded.fetch_add(buf.len() as u64, Ordering::Relaxed);
        }

        return Ok(());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hls_url() {
        assert!(is_hls_url("https://example.com/video.m3u8"));
        assert!(is_hls_url("https://example.com/video.M3U8"));
        assert!(is_hls_url("https://example.com/video.m3u8?token=abc&v=2"));
        assert!(is_hls_url("https://example.com/video.m3u8#section"));
        assert!(!is_hls_url("https://example.com/video.mp4"));
        assert!(!is_hls_url("https://example.com/video.ts"));
    }

    #[test]
    fn test_resolve_url_absolute() {
        assert_eq!(
            resolve_url(
                "https://cdn.example.com/path/manifest.m3u8",
                "https://other.com/video.ts"
            ),
            "https://other.com/video.ts"
        );
    }

    #[test]
    fn test_resolve_url_relative() {
        assert_eq!(
            resolve_url("https://cdn.example.com/path/manifest.m3u8", "segment0.ts"),
            "https://cdn.example.com/path/segment0.ts"
        );
    }

    #[test]
    fn test_resolve_url_relative_subdir() {
        assert_eq!(
            resolve_url(
                "https://cdn.example.com/path/manifest.m3u8",
                "hd/segment0.ts"
            ),
            "https://cdn.example.com/path/hd/segment0.ts"
        );
    }

    #[test]
    fn test_resolve_url_parent() {
        assert_eq!(
            resolve_url(
                "https://cdn.example.com/path/sub/manifest.m3u8",
                "../segment0.ts"
            ),
            "https://cdn.example.com/path/segment0.ts"
        );
    }

    #[test]
    fn test_derive_filename_fmp4() {
        assert_eq!(
            derive_filename("https://example.com/video/xa1wmym.m3u8?sec=abc", true),
            "xa1wmym.mp4"
        );
    }

    #[test]
    fn test_derive_filename_ts() {
        assert_eq!(
            derive_filename("https://example.com/video/xa1wmym.m3u8", false),
            "xa1wmym.ts"
        );
    }

    #[test]
    fn test_derive_filename_manifest_name() {
        assert_eq!(
            derive_filename("https://example.com/video/manifest.m3u8", true),
            "video.mp4"
        );
    }

    #[test]
    fn test_derive_filename_index_name() {
        assert_eq!(
            derive_filename("https://example.com/streams/index.m3u8", false),
            "streams.ts"
        );
    }

    #[test]
    fn test_extract_segments_basic() {
        let playlist = m3u8_rs::MediaPlaylist {
            segments: vec![
                m3u8_rs::MediaSegment {
                    uri: "seg0.ts".into(),
                    duration: 10.0,
                    ..Default::default()
                },
                m3u8_rs::MediaSegment {
                    uri: "seg1.ts".into(),
                    duration: 10.0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let (urls, init) = extract_segments(&playlist, "https://cdn.example.com/path/index.m3u8");

        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://cdn.example.com/path/seg0.ts");
        assert_eq!(urls[1], "https://cdn.example.com/path/seg1.ts");
        assert!(init.is_none());
    }

    #[test]
    fn test_extract_segments_with_init() {
        let playlist = m3u8_rs::MediaPlaylist {
            segments: vec![
                m3u8_rs::MediaSegment {
                    uri: "0.m4s".into(),
                    duration: 5.0,
                    map: Some(m3u8_rs::Map {
                        uri: "init.mp4".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                m3u8_rs::MediaSegment {
                    uri: "1.m4s".into(),
                    duration: 5.0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let (urls, init) = extract_segments(&playlist, "https://cdn.example.com/path/index.m3u8");

        assert_eq!(urls.len(), 2);
        assert_eq!(
            init,
            Some("https://cdn.example.com/path/init.mp4".to_string())
        );
    }

    #[test]
    fn test_hls_data_serde_roundtrip() {
        let data = HlsData {
            segment_urls: vec![
                "https://cdn.example.com/seg0.ts".to_string(),
                "https://cdn.example.com/seg1.ts".to_string(),
            ],
            init_url: Some("https://cdn.example.com/init.mp4".to_string()),
            bandwidth: 5000000,
            resolution: Some("1920x1080".to_string()),
        };

        let json = serde_json::to_string(&data).unwrap();
        let restored: HlsData = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.segment_urls.len(), 2);
        assert_eq!(restored.init_url, data.init_url);
        assert_eq!(restored.bandwidth, 5000000);
        assert_eq!(restored.resolution, Some("1920x1080".to_string()));
    }
}
