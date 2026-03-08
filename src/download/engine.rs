use crate::download::worker::download_segment;
use crate::model::{
    DownloadItem, DownloadStatus, FileCategory, PersistedSegment, ResolvedFileInfo, SegmentInfo,
    SpeedTracker,
};
use crate::settings::DownloadDatabase;
use anyhow::{anyhow, Result};

use futures::FutureExt;
use reqwest::header;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

const MIN_SEGMENT_SIZE: u64 = 2 * 1024 * 1024;

struct SegmentState {
    start: u64,
    end: u64,
    downloaded: Arc<AtomicU64>,
}

struct ManagedDownload {
    id: Uuid,
    url: String,
    filename: String,
    save_path: PathBuf,
    total_size: Option<u64>,
    status: DownloadStatus,
    segments: Vec<SegmentState>,
    category: FileCategory,
    error: Option<String>,
    resumable: bool,
    pause_flag: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
    speed_tracker: SpeedTracker,
    task_handles: Vec<tokio::task::JoinHandle<Result<()>>>,
    /// True when the user clicked Start but was blocked by concurrency limit.
    awaiting_slot: bool,
    headers: HashMap<String, String>,
}

pub struct DownloadEngine {
    state: Mutex<Vec<ManagedDownload>>,
    client: Mutex<reqwest::Client>,
    speed_limit: AtomicU64,
    max_concurrent: AtomicU64,
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadEngine {
    pub fn new() -> Self {
        let client = Self::build_client(None);

        Self {
            state: Mutex::new(Vec::new()),
            client: Mutex::new(client),
            speed_limit: AtomicU64::new(0),
            max_concurrent: AtomicU64::new(3),
        }
    }

    fn build_client(proxy_url: Option<&str>) -> reqwest::Client {
        let mut builder = reqwest::Client::builder()
            .user_agent(format!("Bolt/{}", env!("CARGO_PKG_VERSION")))
            .pool_max_idle_per_host(16)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_nodelay(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(3600));

        if let Some(url) = proxy_url {
            if !url.is_empty() {
                if let Ok(proxy) = reqwest::Proxy::all(url) {
                    builder = builder.proxy(proxy);
                }
            }
        }

        builder.build().expect("Failed to build HTTP client")
    }

    pub fn set_proxy(&self, proxy_url: Option<&str>) {
        let new_client = Self::build_client(proxy_url);
        let mut client = self.client.lock().unwrap();
        *client = new_client;
    }

    pub fn set_speed_limit(&self, bps: u64) {
        self.speed_limit.store(bps, Ordering::Relaxed);
    }

    pub fn get_speed_limit(&self) -> u64 {
        self.speed_limit.load(Ordering::Relaxed)
    }

    pub fn set_max_concurrent(&self, max: u64) {
        self.max_concurrent.store(max, Ordering::Relaxed);
    }

    pub fn get_max_concurrent(&self) -> u64 {
        self.max_concurrent.load(Ordering::Relaxed)
    }

    pub fn restore_downloads(&self, db: &DownloadDatabase) {
        let mut downloads = self.state.lock().unwrap();
        for pd in &db.downloads {
            let segment_states: Vec<SegmentState> = pd
                .segments
                .iter()
                .map(|s| SegmentState {
                    start: s.start,
                    end: s.end,
                    downloaded: Arc::new(AtomicU64::new(s.downloaded)),
                })
                .collect();

            let status = match pd.status {
                DownloadStatus::Downloading | DownloadStatus::Connecting => DownloadStatus::Paused,
                other => other,
            };

            downloads.push(ManagedDownload {
                id: pd.id,
                url: pd.url.clone(),
                filename: pd.filename.clone(),
                save_path: pd.save_path.clone(),
                total_size: pd.total_size,
                status,
                segments: segment_states,
                category: pd.category,
                error: pd.error.clone(),
                resumable: pd.resumable,
                pause_flag: Arc::new(AtomicBool::new(status == DownloadStatus::Paused)),
                cancel_flag: Arc::new(AtomicBool::new(false)),
                speed_tracker: SpeedTracker::new(),
                task_handles: Vec::new(),
                awaiting_slot: false,
                headers: pd.headers.clone(),
            });
        }
    }

    pub fn persist(&self) -> DownloadDatabase {
        let downloads = self.state.lock().unwrap();
        let persisted = downloads
            .iter()
            .map(|dl| {
                let segments: Vec<PersistedSegment> = dl
                    .segments
                    .iter()
                    .map(|s| PersistedSegment {
                        start: s.start,
                        end: s.end,
                        downloaded: s.downloaded.load(Ordering::Relaxed),
                    })
                    .collect();

                let status = match dl.status {
                    DownloadStatus::Downloading | DownloadStatus::Connecting => {
                        DownloadStatus::Paused
                    }
                    other => other,
                };

                crate::model::PersistedDownload {
                    id: dl.id,
                    url: dl.url.clone(),
                    filename: dl.filename.clone(),
                    save_path: dl.save_path.clone(),
                    total_size: dl.total_size,
                    status,
                    segments,
                    category: dl.category,
                    error: dl.error.clone(),
                    resumable: dl.resumable,
                    headers: dl.headers.clone(),
                }
            })
            .collect();

        DownloadDatabase::from_persisted(persisted)
    }

    fn count_downloading(&self, downloads: &[ManagedDownload]) -> usize {
        downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Downloading)
            .count()
    }

    fn per_segment_limit(&self, num_segments: usize) -> u64 {
        let global = self.speed_limit.load(Ordering::Relaxed);
        if global == 0 || num_segments == 0 {
            return 0;
        }
        global / num_segments as u64
    }

    pub async fn add_download(
        self: &Arc<Self>,
        url: String,
        save_dir: PathBuf,
    ) -> Result<DownloadItem> {
        self.add_download_with_headers(url, save_dir, None).await
    }

    pub async fn resolve_file_info(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<ResolvedFileInfo> {
        let client = self.client.lock().unwrap().clone();

        let mut head_req = client.head(url);
        for (k, v) in headers {
            head_req = head_req.header(k.as_str(), v.as_str());
        }
        let response = match head_req.send().await {
            Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => resp,
            _ => {
                let mut get_req = client.get(url);
                for (k, v) in headers {
                    get_req = get_req.header(k.as_str(), v.as_str());
                }
                get_req.send().await?
            }
        };

        if !response.status().is_success()
            && !response.status().is_redirection()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let total_size = response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let resumable = response
            .headers()
            .get(header::ACCEPT_RANGES)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("bytes"))
            .unwrap_or(false);

        let filename = extract_filename(&response, url);

        Ok(ResolvedFileInfo {
            filename,
            total_size,
            resumable,
        })
    }

    pub async fn add_download_resolved(
        self: &Arc<Self>,
        url: String,
        save_dir: PathBuf,
        headers: Option<HashMap<String, String>>,
        info: ResolvedFileInfo,
    ) -> Result<DownloadItem> {
        let hdrs = headers.unwrap_or_default();
        let category = FileCategory::from_filename(&info.filename);

        tokio::fs::create_dir_all(&save_dir).await?;

        let save_path = save_dir.join(&info.filename);
        let id = Uuid::new_v4();

        let num_segments = calc_segment_count(info.total_size, info.resumable);
        let segments = create_segments(info.total_size, num_segments);

        let segment_states: Vec<SegmentState> = segments
            .iter()
            .map(|(start, end)| SegmentState {
                start: *start,
                end: *end,
                downloaded: Arc::new(AtomicU64::new(0)),
            })
            .collect();

        let pause_flag = Arc::new(AtomicBool::new(false));
        let cancel_flag = Arc::new(AtomicBool::new(false));

        let snapshot = build_snapshot(
            id,
            &url,
            &info.filename,
            &save_path,
            info.total_size,
            DownloadStatus::Queued,
            &segment_states,
            0.0,
            category,
            None,
            info.resumable,
        );

        let managed = ManagedDownload {
            id,
            url,
            filename: info.filename,
            save_path,
            total_size: info.total_size,
            status: DownloadStatus::Queued,
            segments: segment_states,
            category,
            error: None,
            resumable: info.resumable,
            pause_flag,
            cancel_flag,
            speed_tracker: SpeedTracker::new(),
            task_handles: Vec::new(),
            awaiting_slot: false,
            headers: hdrs,
        };

        self.state.lock().unwrap().push(managed);
        Ok(snapshot)
    }

    pub async fn add_download_with_headers(
        self: &Arc<Self>,
        url: String,
        save_dir: PathBuf,
        headers: Option<HashMap<String, String>>,
    ) -> Result<DownloadItem> {
        let hdrs = headers.clone().unwrap_or_default();
        let info = self.resolve_file_info(&url, &hdrs).await?;
        self.add_download_resolved(url, save_dir, headers, info)
            .await
    }

    pub async fn start_download(self: &Arc<Self>, id: Uuid) -> Result<()> {
        let (
            url,
            save_path,
            total_size,
            segments_info,
            pause_flag,
            cancel_flag,
            num_segments,
            hdrs,
        ) = {
            let mut downloads = self.state.lock().unwrap();

            let active = self.count_downloading(&downloads);
            let max = self.max_concurrent.load(Ordering::Relaxed) as usize;
            if active >= max {
                if let Some(dl) = downloads.iter_mut().find(|d| d.id == id) {
                    dl.awaiting_slot = true;
                }
                return Err(anyhow!(
                    "Max concurrent downloads reached ({}). Wait for one to finish.",
                    max
                ));
            }

            let dl = downloads
                .iter_mut()
                .find(|d| d.id == id)
                .ok_or_else(|| anyhow!("Download not found"))?;

            if dl.status != DownloadStatus::Queued {
                return Ok(());
            }

            dl.status = DownloadStatus::Downloading;
            dl.error = None;
            dl.awaiting_slot = false;

            let seg_info: Vec<(u64, u64, Arc<AtomicU64>)> = dl
                .segments
                .iter()
                .map(|s| (s.start, s.end, s.downloaded.clone()))
                .collect();

            let n = seg_info.len();

            (
                dl.url.clone(),
                dl.save_path.clone(),
                dl.total_size,
                seg_info,
                dl.pause_flag.clone(),
                dl.cancel_flag.clone(),
                n,
                dl.headers.clone(),
            )
        };

        if let Some(size) = total_size {
            let file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&save_path)
                .await?;
            file.set_len(size).await?;
        }

        let seg_limit = self.per_segment_limit(num_segments);

        let client = self.client.lock().unwrap().clone();
        let mut task_handles = Vec::new();
        for (start, end, downloaded) in segments_info {
            let handle = tokio::spawn(download_segment(
                client.clone(),
                url.clone(),
                save_path.clone(),
                start,
                end,
                downloaded,
                pause_flag.clone(),
                cancel_flag.clone(),
                seg_limit,
                hdrs.clone(),
            ));
            task_handles.push(handle);
        }

        let mut downloads = self.state.lock().unwrap();
        if let Some(dl) = downloads.iter_mut().find(|d| d.id == id) {
            dl.task_handles = task_handles;
        }

        Ok(())
    }

    pub fn pause(&self, id: Uuid) {
        let mut downloads = self.state.lock().unwrap();
        if let Some(dl) = downloads.iter_mut().find(|d| d.id == id) {
            if dl.status == DownloadStatus::Downloading {
                dl.pause_flag.store(true, Ordering::Relaxed);
                dl.status = DownloadStatus::Paused;
                dl.speed_tracker.reset();
            }
        }
    }

    pub async fn resume(self: &Arc<Self>, id: Uuid) -> Result<()> {
        let (url, save_path, segments_info, pause_flag, cancel_flag, num_segments, hdrs) = {
            let mut downloads = self.state.lock().unwrap();

            let active = self.count_downloading(&downloads);
            let max = self.max_concurrent.load(Ordering::Relaxed) as usize;
            if active >= max {
                return Err(anyhow!(
                    "Max concurrent downloads reached ({}). Wait for one to finish.",
                    max
                ));
            }

            let dl = downloads
                .iter_mut()
                .find(|d| d.id == id)
                .ok_or_else(|| anyhow!("Download not found"))?;

            if dl.status != DownloadStatus::Paused {
                return Ok(());
            }

            dl.pause_flag.store(false, Ordering::Relaxed);
            dl.status = DownloadStatus::Downloading;
            dl.speed_tracker.reset();

            for handle in dl.task_handles.drain(..) {
                handle.abort();
            }

            let seg_info: Vec<(u64, u64, Arc<AtomicU64>)> = dl
                .segments
                .iter()
                .map(|s| (s.start, s.end, s.downloaded.clone()))
                .collect();

            let n = seg_info.len();

            (
                dl.url.clone(),
                dl.save_path.clone(),
                seg_info,
                dl.pause_flag.clone(),
                dl.cancel_flag.clone(),
                n,
                dl.headers.clone(),
            )
        };

        let seg_limit = self.per_segment_limit(num_segments);

        let client = self.client.lock().unwrap().clone();
        let mut new_handles = Vec::new();
        for (start, end, downloaded) in segments_info {
            let done = downloaded.load(Ordering::Relaxed);
            if end != u64::MAX && start + done >= end {
                continue;
            }

            let handle = tokio::spawn(download_segment(
                client.clone(),
                url.clone(),
                save_path.clone(),
                start,
                end,
                downloaded,
                pause_flag.clone(),
                cancel_flag.clone(),
                seg_limit,
                hdrs.clone(),
            ));
            new_handles.push(handle);
        }

        let mut downloads = self.state.lock().unwrap();
        if let Some(dl) = downloads.iter_mut().find(|d| d.id == id) {
            dl.task_handles = new_handles;
        }

        Ok(())
    }

    pub fn cancel(&self, id: Uuid) {
        let mut downloads = self.state.lock().unwrap();
        if let Some(dl) = downloads.iter_mut().find(|d| d.id == id) {
            dl.cancel_flag.store(true, Ordering::Relaxed);
            dl.status = DownloadStatus::Cancelled;
            dl.speed_tracker.reset();
            for handle in dl.task_handles.drain(..) {
                handle.abort();
            }
        }
    }

    pub fn remove(&self, id: Uuid) {
        let mut downloads = self.state.lock().unwrap();
        if let Some(pos) = downloads.iter().position(|d| d.id == id) {
            let dl = &downloads[pos];
            dl.cancel_flag.store(true, Ordering::Relaxed);
            for handle in &dl.task_handles {
                handle.abort();
            }
            downloads.remove(pos);
        }
    }

    pub fn clear_completed(&self) {
        let mut downloads = self.state.lock().unwrap();
        downloads.retain(|d| d.status != DownloadStatus::Completed);
    }

    pub async fn retry(self: &Arc<Self>, id: Uuid) -> Result<DownloadItem> {
        let (url, save_dir, hdrs) = {
            let downloads = self.state.lock().unwrap();
            let dl = downloads
                .iter()
                .find(|d| d.id == id)
                .ok_or_else(|| anyhow!("Download not found"))?;
            let save_dir = dl
                .save_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf();
            (dl.url.clone(), save_dir, dl.headers.clone())
        };

        self.remove(id);
        let h = if hdrs.is_empty() { None } else { Some(hdrs) };
        self.add_download_with_headers(url, save_dir, h).await
    }

    /// Auto-start queued downloads that were blocked by concurrency limit.
    pub fn auto_start_queued(&self) -> Vec<Uuid> {
        let downloads = self.state.lock().unwrap();
        let active = self.count_downloading(&downloads);
        let max = self.max_concurrent.load(Ordering::Relaxed) as usize;

        if active >= max {
            return Vec::new();
        }

        let slots = max - active;
        downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Queued && d.awaiting_slot)
            .take(slots)
            .map(|d| d.id)
            .collect()
    }

    pub fn get_failed_ids(&self) -> Vec<Uuid> {
        let downloads = self.state.lock().unwrap();
        downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Failed)
            .map(|d| d.id)
            .collect()
    }

    pub fn get_queued_ids(&self) -> Vec<Uuid> {
        let downloads = self.state.lock().unwrap();
        downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Queued)
            .map(|d| d.id)
            .collect()
    }

    #[cfg(test)]
    pub fn mark_awaiting_slot(&self, id: Uuid) {
        let mut downloads = self.state.lock().unwrap();
        if let Some(dl) = downloads.iter_mut().find(|d| d.id == id) {
            dl.awaiting_slot = true;
        }
    }

    pub fn update_state(&self) {
        let mut downloads = self.state.lock().unwrap();
        for dl in downloads.iter_mut() {
            if dl.status != DownloadStatus::Downloading {
                continue;
            }

            let raw_downloaded: u64 = dl
                .segments
                .iter()
                .map(|s| s.downloaded.load(Ordering::Relaxed))
                .sum();

            let capped = match dl.total_size {
                Some(total) => raw_downloaded.min(total),
                None => raw_downloaded,
            };

            dl.speed_tracker.record(capped);

            let all_finished = dl.task_handles.iter().all(|h| h.is_finished());

            if all_finished && !dl.task_handles.is_empty() {
                let mut any_task_error = false;

                for handle in dl.task_handles.drain(..) {
                    match handle.now_or_never() {
                        Some(Ok(Ok(()))) => {}
                        Some(Ok(Err(e))) => {
                            any_task_error = true;
                            dl.error = Some(e.to_string());
                        }
                        Some(Err(_join_err)) => {
                            any_task_error = true;
                            dl.error = Some("Worker task panicked".to_string());
                        }
                        None => {}
                    }
                }

                let bytes_incomplete = dl.total_size.is_some()
                    && dl.segments.iter().any(|s| {
                        if s.end == u64::MAX {
                            return false;
                        }
                        let expected = s.end - s.start;
                        s.downloaded.load(Ordering::Relaxed) < expected
                    });

                if any_task_error || bytes_incomplete {
                    dl.status = DownloadStatus::Failed;
                    if dl.error.is_none() {
                        dl.error = Some("Download incomplete".to_string());
                    }
                } else {
                    dl.status = DownloadStatus::Completed;
                }
                dl.speed_tracker.reset();
                continue;
            }

            dl.task_handles.retain(|h| !h.is_finished());
        }
    }

    pub fn get_ui_state(&self) -> (Vec<DownloadItem>, f64, (usize, usize, usize, usize, usize)) {
        let downloads = self.state.lock().unwrap();

        let mut snapshots = Vec::with_capacity(downloads.len());
        let mut total_speed = 0.0;
        let mut active = 0usize;
        let mut completed = 0usize;
        let mut paused = 0usize;
        let mut failed = 0usize;

        for dl in downloads.iter() {
            let speed = dl.speed_tracker.speed();

            if dl.status == DownloadStatus::Downloading {
                total_speed += speed;
            }

            match dl.status {
                DownloadStatus::Queued
                | DownloadStatus::Connecting
                | DownloadStatus::Downloading => active += 1,
                DownloadStatus::Completed => completed += 1,
                DownloadStatus::Paused => paused += 1,
                DownloadStatus::Failed | DownloadStatus::Cancelled => failed += 1,
            }

            snapshots.push(build_snapshot(
                dl.id,
                &dl.url,
                &dl.filename,
                &dl.save_path,
                dl.total_size,
                dl.status,
                &dl.segments,
                speed,
                dl.category,
                dl.error.clone(),
                dl.resumable,
            ));
        }

        let total = downloads.len();
        (
            snapshots,
            total_speed,
            (total, active, completed, paused, failed),
        )
    }
}

fn calc_segment_count(total_size: Option<u64>, resumable: bool) -> usize {
    if !resumable {
        return 1;
    }
    let size = match total_size {
        Some(s) if s > MIN_SEGMENT_SIZE => s,
        _ => return 1,
    };

    let by_size = (size / MIN_SEGMENT_SIZE) as usize;

    let target = if size < 5 * 1024 * 1024 {
        1
    } else if size < 20 * 1024 * 1024 {
        2
    } else if size < 50 * 1024 * 1024 {
        4
    } else if size < 200 * 1024 * 1024 {
        6
    } else {
        8
    };

    target.min(by_size).max(1)
}

fn create_segments(total_size: Option<u64>, num_segments: usize) -> Vec<(u64, u64)> {
    let total = match total_size {
        Some(s) if s > 0 => s,
        _ => return vec![(0, u64::MAX)],
    };

    let segment_size = total / num_segments as u64;
    let mut segments = Vec::new();

    for i in 0..num_segments {
        let start = i as u64 * segment_size;
        let end = if i == num_segments - 1 {
            total
        } else {
            (i as u64 + 1) * segment_size
        };
        segments.push((start, end));
    }

    segments
}

fn extract_filename(response: &reqwest::Response, url: &str) -> String {
    if let Some(cd) = response.headers().get(header::CONTENT_DISPOSITION) {
        if let Ok(cd_str) = cd.to_str() {
            if let Some(fname) = parse_content_disposition(cd_str) {
                return fname;
            }
        }
    }

    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(mut segments) = parsed.path_segments() {
            if let Some(last) = segments.next_back() {
                let decoded = urldecode(last);
                if !decoded.is_empty() && decoded != "/" {
                    return decoded;
                }
            }
        }
    }

    format!("download_{}", Uuid::new_v4().as_simple())
}

fn parse_content_disposition(header: &str) -> Option<String> {
    for part in header.split(';') {
        let part = part.trim();
        if part.starts_with("filename*=") {
            let value = part.strip_prefix("filename*=")?;
            if let Some(encoded) = value.split("''").nth(1) {
                return Some(urldecode(encoded));
            }
        }
        if part.starts_with("filename=") {
            let value = part.strip_prefix("filename=")?;
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

fn urldecode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn build_snapshot(
    id: Uuid,
    url: &str,
    filename: &str,
    save_path: &std::path::Path,
    total_size: Option<u64>,
    status: DownloadStatus,
    segments: &[SegmentState],
    speed: f64,
    category: FileCategory,
    error: Option<String>,
    resumable: bool,
) -> DownloadItem {
    let raw_downloaded: u64 = segments
        .iter()
        .map(|s| s.downloaded.load(Ordering::Relaxed))
        .sum();

    let total_downloaded = match total_size {
        Some(total) => raw_downloaded.min(total),
        None => raw_downloaded,
    };

    let segment_infos: Vec<SegmentInfo> = segments
        .iter()
        .enumerate()
        .map(|(i, s)| SegmentInfo {
            index: i,
            start: s.start,
            end: s.end,
            downloaded: s.downloaded.load(Ordering::Relaxed),
        })
        .collect();

    DownloadItem {
        id,
        url: url.to_string(),
        filename: filename.to_string(),
        save_path: save_path.to_path_buf(),
        total_size,
        downloaded: total_downloaded,
        status,
        segments: segment_infos,
        speed,
        category,
        error,
        resumable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_count_not_resumable() {
        assert_eq!(calc_segment_count(Some(500 * 1024 * 1024), false), 1);
    }

    #[test]
    fn segment_count_unknown_size() {
        assert_eq!(calc_segment_count(None, true), 1);
    }

    #[test]
    fn segment_count_tiny_file() {
        assert_eq!(calc_segment_count(Some(1024), true), 1);
    }

    #[test]
    fn segment_count_small_file() {
        assert_eq!(calc_segment_count(Some(3 * 1024 * 1024), true), 1);
    }

    #[test]
    fn segment_count_medium_file() {
        assert_eq!(calc_segment_count(Some(25 * 1024 * 1024), true), 4);
    }

    #[test]
    fn segment_count_large_file() {
        assert_eq!(calc_segment_count(Some(100 * 1024 * 1024), true), 6);
    }

    #[test]
    fn segment_count_very_large_file() {
        assert_eq!(calc_segment_count(Some(500 * 1024 * 1024), true), 8);
    }

    #[test]
    fn segments_unknown_size() {
        let segs = create_segments(None, 4);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], (0, u64::MAX));
    }

    #[test]
    fn segments_zero_size() {
        let segs = create_segments(Some(0), 4);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], (0, u64::MAX));
    }

    #[test]
    fn segments_single() {
        let segs = create_segments(Some(1000), 1);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], (0, 1000));
    }

    #[test]
    fn segments_multiple_cover_full_range() {
        let total = 1024 * 1024;
        let segs = create_segments(Some(total), 4);
        assert_eq!(segs.len(), 4);
        assert_eq!(segs[0].0, 0);
        assert_eq!(segs.last().unwrap().1, total);

        for i in 1..segs.len() {
            assert_eq!(segs[i].0, segs[i - 1].1);
        }
    }

    #[test]
    fn segments_no_gaps_or_overlaps() {
        let total = 10 * 1024 * 1024;
        let segs = create_segments(Some(total), 8);

        let mut covered = 0u64;
        for (start, end) in &segs {
            assert_eq!(*start, covered);
            assert!(*end > *start);
            covered = *end;
        }
        assert_eq!(covered, total);
    }

    #[test]
    fn content_disposition_simple_filename() {
        let result = parse_content_disposition("attachment; filename=\"report.pdf\"");
        assert_eq!(result, Some("report.pdf".to_string()));
    }

    #[test]
    fn content_disposition_unquoted() {
        let result = parse_content_disposition("attachment; filename=report.pdf");
        assert_eq!(result, Some("report.pdf".to_string()));
    }

    #[test]
    fn content_disposition_encoded() {
        let result = parse_content_disposition("attachment; filename*=UTF-8''my%20file%20name.pdf");
        assert_eq!(result, Some("my file name.pdf".to_string()));
    }

    #[test]
    fn content_disposition_encoded_preferred() {
        let result = parse_content_disposition(
            "attachment; filename*=UTF-8''encoded%20name.pdf; filename=\"fallback.pdf\"",
        );
        assert_eq!(result, Some("encoded name.pdf".to_string()));
    }

    #[test]
    fn content_disposition_no_filename() {
        let result = parse_content_disposition("inline");
        assert_eq!(result, None);
    }

    #[test]
    fn urldecode_plain() {
        assert_eq!(urldecode("hello"), "hello");
    }

    #[test]
    fn urldecode_spaces() {
        assert_eq!(urldecode("hello+world"), "hello world");
    }

    #[test]
    fn urldecode_percent() {
        assert_eq!(urldecode("hello%20world"), "hello world");
    }

    #[test]
    fn urldecode_mixed() {
        assert_eq!(urldecode("file%20name+here.zip"), "file name here.zip");
    }

    #[test]
    fn urldecode_special_chars() {
        assert_eq!(urldecode("%2Fpath%2Fto%2Ffile"), "/path/to/file");
    }

    #[test]
    fn engine_default() {
        let engine = DownloadEngine::default();
        let (snapshots, speed, counts) = engine.get_ui_state();
        assert!(snapshots.is_empty());
        assert!((speed - 0.0).abs() < 0.01);
        assert_eq!(counts, (0, 0, 0, 0, 0));
    }

    #[test]
    fn engine_clear_completed_empty() {
        let engine = DownloadEngine::new();
        engine.clear_completed();
        let (snapshots, _, _) = engine.get_ui_state();
        assert!(snapshots.is_empty());
    }

    #[test]
    fn engine_remove_nonexistent() {
        let engine = DownloadEngine::new();
        engine.remove(Uuid::new_v4());
        let (snapshots, _, _) = engine.get_ui_state();
        assert!(snapshots.is_empty());
    }

    #[test]
    fn engine_pause_nonexistent() {
        let engine = DownloadEngine::new();
        engine.pause(Uuid::new_v4());
    }

    #[test]
    fn engine_cancel_nonexistent() {
        let engine = DownloadEngine::new();
        engine.cancel(Uuid::new_v4());
    }

    #[test]
    fn engine_speed_limit() {
        let engine = DownloadEngine::new();
        assert_eq!(engine.get_speed_limit(), 0);
        engine.set_speed_limit(1_000_000);
        assert_eq!(engine.get_speed_limit(), 1_000_000);
    }

    #[test]
    fn engine_max_concurrent() {
        let engine = DownloadEngine::new();
        assert_eq!(engine.get_max_concurrent(), 3);
        engine.set_max_concurrent(5);
        assert_eq!(engine.get_max_concurrent(), 5);
    }

    #[test]
    fn engine_per_segment_limit() {
        let engine = DownloadEngine::new();
        assert_eq!(engine.per_segment_limit(4), 0);
        engine.set_speed_limit(1_000_000);
        assert_eq!(engine.per_segment_limit(4), 250_000);
        assert_eq!(engine.per_segment_limit(0), 0);
    }

    #[test]
    fn engine_persist_empty() {
        let engine = DownloadEngine::new();
        let db = engine.persist();
        assert!(db.downloads.is_empty());
    }

    #[test]
    fn engine_restore_and_persist_roundtrip() {
        use crate::model::{PersistedDownload, PersistedSegment};
        use crate::settings::DownloadDatabase;

        let id = Uuid::new_v4();
        let db = DownloadDatabase::from_persisted(vec![PersistedDownload {
            id,
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            save_path: PathBuf::from("/tmp/file.zip"),
            total_size: Some(1024),
            status: DownloadStatus::Paused,
            segments: vec![PersistedSegment {
                start: 0,
                end: 1024,
                downloaded: 512,
            }],
            category: FileCategory::Archive,
            error: None,
            resumable: true,
            headers: HashMap::new(),
        }]);

        let engine = DownloadEngine::new();
        engine.restore_downloads(&db);

        let (snapshots, _, counts) = engine.get_ui_state();
        assert_eq!(counts.0, 1);
        assert_eq!(snapshots[0].id, id);
        assert_eq!(snapshots[0].filename, "file.zip");
        assert_eq!(snapshots[0].status, DownloadStatus::Paused);
        assert_eq!(snapshots[0].downloaded, 512);

        let persisted = engine.persist();
        assert_eq!(persisted.downloads.len(), 1);
        assert_eq!(persisted.downloads[0].id, id);
        assert_eq!(persisted.downloads[0].segments[0].downloaded, 512);
    }

    #[test]
    fn engine_restore_downloading_becomes_paused() {
        use crate::model::{PersistedDownload, PersistedSegment};
        use crate::settings::DownloadDatabase;

        let db = DownloadDatabase::from_persisted(vec![PersistedDownload {
            id: Uuid::new_v4(),
            url: "https://example.com/f.bin".to_string(),
            filename: "f.bin".to_string(),
            save_path: PathBuf::from("/tmp/f.bin"),
            total_size: Some(2048),
            status: DownloadStatus::Downloading,
            segments: vec![PersistedSegment {
                start: 0,
                end: 2048,
                downloaded: 1000,
            }],
            category: FileCategory::Other,
            error: None,
            resumable: true,
            headers: HashMap::new(),
        }]);

        let engine = DownloadEngine::new();
        engine.restore_downloads(&db);

        let (snapshots, _, _) = engine.get_ui_state();
        assert_eq!(snapshots[0].status, DownloadStatus::Paused);
    }

    #[test]
    fn engine_auto_start_queued_respects_limit() {
        use crate::model::{PersistedDownload, PersistedSegment};
        use crate::settings::DownloadDatabase;

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let db = DownloadDatabase::from_persisted(vec![
            PersistedDownload {
                id: id1,
                url: "https://example.com/a.bin".to_string(),
                filename: "a.bin".to_string(),
                save_path: PathBuf::from("/tmp/a.bin"),
                total_size: Some(1024),
                status: DownloadStatus::Queued,
                segments: vec![PersistedSegment {
                    start: 0,
                    end: 1024,
                    downloaded: 0,
                }],
                category: FileCategory::Other,
                error: None,
                resumable: false,
                headers: HashMap::new(),
            },
            PersistedDownload {
                id: id2,
                url: "https://example.com/b.bin".to_string(),
                filename: "b.bin".to_string(),
                save_path: PathBuf::from("/tmp/b.bin"),
                total_size: Some(1024),
                status: DownloadStatus::Queued,
                segments: vec![PersistedSegment {
                    start: 0,
                    end: 1024,
                    downloaded: 0,
                }],
                category: FileCategory::Other,
                error: None,
                resumable: false,
                headers: HashMap::new(),
            },
        ]);

        let engine = DownloadEngine::new();
        engine.set_max_concurrent(1);
        engine.restore_downloads(&db);

        // Downloads must have awaiting_slot=true (user tried to start them)
        engine.mark_awaiting_slot(id1);
        engine.mark_awaiting_slot(id2);

        let auto = engine.auto_start_queued();
        assert_eq!(auto.len(), 1);
    }

    #[test]
    fn engine_auto_start_skips_fresh_queued() {
        use crate::model::{PersistedDownload, PersistedSegment};
        use crate::settings::DownloadDatabase;

        let db = DownloadDatabase::from_persisted(vec![PersistedDownload {
            id: Uuid::new_v4(),
            url: "https://example.com/f.bin".to_string(),
            filename: "f.bin".to_string(),
            save_path: PathBuf::from("/tmp/f.bin"),
            total_size: Some(1024),
            status: DownloadStatus::Queued,
            segments: vec![PersistedSegment {
                start: 0,
                end: 1024,
                downloaded: 0,
            }],
            category: FileCategory::Other,
            error: None,
            resumable: false,
            headers: HashMap::new(),
        }]);

        let engine = DownloadEngine::new();
        engine.restore_downloads(&db);

        let auto = engine.auto_start_queued();
        assert!(
            auto.is_empty(),
            "fresh queued downloads should not auto-start"
        );
    }

    #[test]
    fn engine_get_failed_ids() {
        use crate::model::{PersistedDownload, PersistedSegment};
        use crate::settings::DownloadDatabase;

        let id_failed = Uuid::new_v4();
        let id_paused = Uuid::new_v4();
        let db = DownloadDatabase::from_persisted(vec![
            PersistedDownload {
                id: id_failed,
                url: "https://example.com/a.bin".to_string(),
                filename: "a.bin".to_string(),
                save_path: PathBuf::from("/tmp/a.bin"),
                total_size: Some(1024),
                status: DownloadStatus::Failed,
                segments: vec![PersistedSegment {
                    start: 0,
                    end: 1024,
                    downloaded: 100,
                }],
                category: FileCategory::Other,
                error: Some("timeout".to_string()),
                resumable: true,
                headers: HashMap::new(),
            },
            PersistedDownload {
                id: id_paused,
                url: "https://example.com/b.bin".to_string(),
                filename: "b.bin".to_string(),
                save_path: PathBuf::from("/tmp/b.bin"),
                total_size: Some(2048),
                status: DownloadStatus::Paused,
                segments: vec![PersistedSegment {
                    start: 0,
                    end: 2048,
                    downloaded: 512,
                }],
                category: FileCategory::Other,
                error: None,
                resumable: true,
                headers: HashMap::new(),
            },
        ]);

        let engine = DownloadEngine::new();
        engine.restore_downloads(&db);

        let failed = engine.get_failed_ids();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0], id_failed);
    }

    #[test]
    fn engine_get_failed_ids_empty() {
        let engine = DownloadEngine::new();
        assert!(engine.get_failed_ids().is_empty());
    }
}
