use anyhow::Result;
use futures::StreamExt;
use reqwest::header;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

const MAX_RETRIES: u32 = 3;
const WRITE_BUF_SIZE: usize = 256 * 1024;

pub async fn download_segment(
    client: reqwest::Client,
    url: String,
    file_path: PathBuf,
    start: u64,
    end: u64,
    downloaded: Arc<AtomicU64>,
    pause_flag: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<()> {
    for attempt in 0..=MAX_RETRIES {
        if cancel_flag.load(Ordering::Relaxed) {
            return Ok(());
        }

        match try_download_segment(
            &client,
            &url,
            &file_path,
            start,
            end,
            &downloaded,
            &pause_flag,
            &cancel_flag,
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(_) if cancel_flag.load(Ordering::Relaxed) => return Ok(()),
            Err(_) if pause_flag.load(Ordering::Relaxed) => return Ok(()),
            Err(e) if attempt == MAX_RETRIES => return Err(e),
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(3u64.pow(attempt))).await;
            }
        }
    }
    Ok(())
}

async fn flush_buf(
    file: &mut tokio::fs::File,
    buf: &mut Vec<u8>,
    downloaded: &AtomicU64,
) -> Result<()> {
    if !buf.is_empty() {
        file.write_all(buf).await?;
        downloaded.fetch_add(buf.len() as u64, Ordering::Relaxed);
        buf.clear();
    }
    Ok(())
}

async fn try_download_segment(
    client: &reqwest::Client,
    url: &str,
    file_path: &PathBuf,
    start: u64,
    end: u64,
    downloaded: &Arc<AtomicU64>,
    pause_flag: &Arc<AtomicBool>,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<()> {
    let already_downloaded = downloaded.load(Ordering::Relaxed);
    let actual_start = start + already_downloaded;

    if actual_start >= end && end != u64::MAX {
        return Ok(());
    }

    let mut request = client.get(url);

    if end != u64::MAX {
        request = request.header(
            header::RANGE,
            format!("bytes={}-{}", actual_start, end - 1),
        );
    }

    let response = request.send().await?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
    {
        return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)
        .await?;

    if end != u64::MAX {
        file.seek(std::io::SeekFrom::Start(actual_start)).await?;
    }

    let mut stream = response.bytes_stream();
    let mut buf = Vec::with_capacity(WRITE_BUF_SIZE);

    while let Some(chunk_result) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) || pause_flag.load(Ordering::Relaxed) {
            flush_buf(&mut file, &mut buf, downloaded).await?;
            return Ok(());
        }

        let chunk = chunk_result?;
        buf.extend_from_slice(&chunk);

        if buf.len() >= WRITE_BUF_SIZE {
            flush_buf(&mut file, &mut buf, downloaded).await?;
        }
    }

    flush_buf(&mut file, &mut buf, downloaded).await?;
    Ok(())
}
