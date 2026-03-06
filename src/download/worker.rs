use anyhow::Result;
use futures::StreamExt;
use reqwest::header;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

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
    let already_downloaded = downloaded.load(Ordering::Relaxed);
    let actual_start = start + already_downloaded;

    if actual_start >= end && end != u64::MAX {
        return Ok(());
    }

    let mut request = client.get(&url);

    if end != u64::MAX {
        request = request.header(
            header::RANGE,
            format!("bytes={}-{}", actual_start, end - 1),
        );
    }

    let response = request.send().await?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
    {
        return Err(anyhow::anyhow!(
            "HTTP error: {}",
            response.status()
        ));
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_path)
        .await?;

    if end != u64::MAX {
        file.seek(std::io::SeekFrom::Start(actual_start)).await?;
    }

    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            return Ok(());
        }

        while pause_flag.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if cancel_flag.load(Ordering::Relaxed) {
                return Ok(());
            }
        }

        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    }

    file.flush().await?;
    Ok(())
}
