use bolt::download::hls::{download_hls, resolve_hls};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Bolt/test")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
}

fn download_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Bolt/test")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap()
}

// ─── resolve_hls tests against real public streams ──────────────────────

#[tokio::test]
async fn resolve_fmp4_master_playlist() {
    let client = test_client();
    let url =
        "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8";

    let (filename, data) = resolve_hls(&client, url, &HashMap::new())
        .await
        .expect("Failed to resolve Apple fMP4 test stream");

    assert!(
        filename.ends_with(".mp4"),
        "fMP4 stream filename should end with .mp4, got: {}",
        filename
    );
    assert!(
        data.init_url.is_some(),
        "fMP4 stream should have an init segment URL"
    );
    assert!(
        !data.segment_urls.is_empty(),
        "Should have resolved segment URLs"
    );
    assert!(
        data.bandwidth > 0,
        "Should have selected a variant with bandwidth > 0"
    );
    assert!(
        data.resolution.is_some(),
        "Best variant should have a resolution"
    );

    println!("  filename:    {}", filename);
    println!("  bandwidth:   {}", data.bandwidth);
    println!(
        "  resolution:  {}",
        data.resolution.as_deref().unwrap_or("-")
    );
    println!("  init_url:    {}", data.init_url.as_deref().unwrap_or("-"));
    println!("  segments:    {}", data.segment_urls.len());
    println!(
        "  first seg:   {}",
        data.segment_urls
            .first()
            .map(|u| &u[..u.len().min(100)])
            .unwrap_or("-")
    );
}

#[tokio::test]
async fn resolve_ts_media_playlist() {
    let client = test_client();
    let url = "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8";

    let (filename, data) = resolve_hls(&client, url, &HashMap::new())
        .await
        .expect("Failed to resolve Big Buck Bunny test stream");

    assert!(
        filename.ends_with(".mp4") || filename.ends_with(".ts"),
        "Filename should end with .mp4 or .ts, got: {}",
        filename
    );
    assert!(
        !data.segment_urls.is_empty(),
        "Should have resolved segment URLs"
    );

    println!("  filename:    {}", filename);
    println!("  init_url:    {:?}", data.init_url);
    println!("  segments:    {}", data.segment_urls.len());
}

#[tokio::test]
async fn resolve_selects_highest_bandwidth() {
    let client = test_client();
    let url =
        "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8";

    let (_, data) = resolve_hls(&client, url, &HashMap::new())
        .await
        .expect("Failed to resolve");

    // Apple's test stream has variants up to ~8Mbps; the best should be > 1Mbps
    assert!(
        data.bandwidth > 1_000_000,
        "Should select a high-bandwidth variant, got: {}",
        data.bandwidth
    );
}

#[tokio::test]
async fn resolve_segments_are_absolute_urls() {
    let client = test_client();
    let url =
        "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8";

    let (_, data) = resolve_hls(&client, url, &HashMap::new())
        .await
        .expect("Failed to resolve");

    for seg_url in &data.segment_urls {
        assert!(
            seg_url.starts_with("https://") || seg_url.starts_with("http://"),
            "Segment URL should be absolute, got: {}",
            seg_url
        );
    }

    if let Some(ref init) = data.init_url {
        assert!(
            init.starts_with("https://") || init.starts_with("http://"),
            "Init URL should be absolute, got: {}",
            init
        );
    }
}

// ─── Small HLS download test ────────────────────────────────────────────

#[tokio::test]
async fn download_small_fmp4_stream() {
    let resolve_client = test_client();
    let url =
        "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8";

    let (filename, mut data) = resolve_hls(&resolve_client, url, &HashMap::new())
        .await
        .expect("Failed to resolve");

    // Limit to init + first 2 segments to keep the test fast
    data.segment_urls.truncate(2);

    let tmp_dir = std::env::temp_dir().join("bolt_hls_test");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let save_path = tmp_dir.join(&filename);

    let downloaded = Arc::new(AtomicU64::new(0));
    let pause = Arc::new(AtomicBool::new(false));
    let cancel = Arc::new(AtomicBool::new(false));

    download_hls(
        download_client(),
        data.clone(),
        save_path.clone(),
        downloaded.clone(),
        pause,
        cancel,
        0,
        HashMap::new(),
    )
    .await
    .expect("HLS download failed");

    let bytes_written = downloaded.load(Ordering::Relaxed);
    assert!(bytes_written > 0, "Should have downloaded some bytes");

    let meta = std::fs::metadata(&save_path).expect("Output file should exist");
    assert!(meta.len() > 0, "Output file should be non-empty");
    assert_eq!(
        meta.len(),
        bytes_written,
        "File size should match downloaded counter"
    );

    // fMP4: file should start with an MP4 box (first 4 bytes = size, next 4 = 'ftyp' or 'moov' or 'moof')
    let header = std::fs::read(&save_path).unwrap();
    if data.init_url.is_some() && header.len() >= 8 {
        let box_type = std::str::from_utf8(&header[4..8]).unwrap_or("");
        assert!(
            box_type == "ftyp" || box_type == "styp" || box_type == "moof" || box_type == "moov",
            "fMP4 file should start with a valid MP4 box, got: {:?}",
            box_type
        );
    }

    println!("  saved to:    {}", save_path.display());
    println!("  file size:   {} bytes", meta.len());

    // Cleanup
    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_dir(&tmp_dir);
}

#[tokio::test]
async fn download_honours_cancel_flag() {
    let resolve_client = test_client();
    let url =
        "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8";

    let (filename, data) = resolve_hls(&resolve_client, url, &HashMap::new())
        .await
        .expect("Failed to resolve");

    let tmp_dir = std::env::temp_dir().join("bolt_hls_cancel_test");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let save_path = tmp_dir.join(&filename);

    let downloaded = Arc::new(AtomicU64::new(0));
    let pause = Arc::new(AtomicBool::new(false));
    let cancel = Arc::new(AtomicBool::new(true)); // pre-cancelled

    download_hls(
        download_client(),
        data,
        save_path.clone(),
        downloaded.clone(),
        pause,
        cancel,
        0,
        HashMap::new(),
    )
    .await
    .expect("Cancelled download should not error");

    let bytes = downloaded.load(Ordering::Relaxed);
    assert_eq!(bytes, 0, "Cancelled download should write 0 bytes");

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_dir(&tmp_dir);
}

// ─── IPC simulation test ────────────────────────────────────────────────

#[tokio::test]
async fn ipc_accepts_hls_url() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    let stream = match TcpStream::connect("127.0.0.1:9817").await {
        Ok(s) => s,
        Err(_) => {
            println!("  SKIPPED: Bolt is not running on port 9817");
            return;
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let msg = serde_json::json!({
        "url": "https://devstreaming-cdn.apple.com/videos/streaming/examples/img_bipbop_adv_example_fmp4/master.m3u8",
        "filename": "test_hls_video.mp4"
    });

    let mut payload = serde_json::to_string(&msg).unwrap();
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await.unwrap();

    let response_line = lines
        .next_line()
        .await
        .unwrap()
        .expect("Should get a response");
    let resp: serde_json::Value = serde_json::from_str(&response_line).unwrap();

    assert_eq!(
        resp["status"].as_str(),
        Some("ok"),
        "IPC should accept the m3u8 URL, got: {}",
        resp
    );

    println!("  IPC response: {}", resp);
}
