use crate::download::worker::download_segment;
use crate::model::{DownloadItem, DownloadStatus, FileCategory, SegmentInfo, SpeedTracker};
use anyhow::{anyhow, Result};
use reqwest::header;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const MIN_SEGMENT_SIZE: u64 = 2 * 1024 * 1024; // 2 MB minimum per segment

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
}

pub struct DownloadEngine {
    state: Mutex<Vec<ManagedDownload>>,
    client: reqwest::Client,
}

impl DownloadEngine {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Bolt/0.1.0")
            .build()
            .unwrap_or_default();

        Self {
            state: Mutex::new(Vec::new()),
            client,
        }
    }

    pub async fn add_download(
        self: &Arc<Self>,
        url: String,
        save_dir: PathBuf,
    ) -> Result<DownloadItem> {
        let response = match self.client.head(&url).send().await {
            Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => resp,
            _ => self.client.get(&url).send().await?,
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

        let filename = extract_filename(&response, &url);
        let category = FileCategory::from_filename(&filename);

        tokio::fs::create_dir_all(&save_dir).await?;

        let save_path = save_dir.join(&filename);
        let id = Uuid::new_v4();

        let num_segments = calc_segment_count(total_size, resumable);

        let segments = create_segments(total_size, num_segments);

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
            &filename,
            &save_path,
            total_size,
            DownloadStatus::Queued,
            &segment_states,
            0.0,
            category,
            None,
            resumable,
        );

        let managed = ManagedDownload {
            id,
            url,
            filename,
            save_path,
            total_size,
            status: DownloadStatus::Queued,
            segments: segment_states,
            category,
            error: None,
            resumable,
            pause_flag,
            cancel_flag,
            speed_tracker: SpeedTracker::new(),
            task_handles: Vec::new(),
        };

        self.state.lock().unwrap().push(managed);
        Ok(snapshot)
    }

    pub async fn start_download(self: &Arc<Self>, id: Uuid) -> Result<()> {
        let (url, save_path, total_size, segments_info, pause_flag, cancel_flag) = {
            let mut downloads = self.state.lock().unwrap();
            let dl = downloads
                .iter_mut()
                .find(|d| d.id == id)
                .ok_or_else(|| anyhow!("Download not found"))?;

            if dl.status != DownloadStatus::Queued {
                return Ok(());
            }

            dl.status = DownloadStatus::Downloading;

            let seg_info: Vec<(u64, u64, Arc<AtomicU64>)> = dl
                .segments
                .iter()
                .map(|s| (s.start, s.end, s.downloaded.clone()))
                .collect();

            (
                dl.url.clone(),
                dl.save_path.clone(),
                dl.total_size,
                seg_info,
                dl.pause_flag.clone(),
                dl.cancel_flag.clone(),
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

        let mut task_handles = Vec::new();
        for (start, end, downloaded) in segments_info {
            let handle = tokio::spawn(download_segment(
                self.client.clone(),
                url.clone(),
                save_path.clone(),
                start,
                end,
                downloaded,
                pause_flag.clone(),
                cancel_flag.clone(),
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
        let (url, save_path, segments_info, pause_flag, cancel_flag) = {
            let mut downloads = self.state.lock().unwrap();
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
                .map(|s| {
                    let downloaded = s.downloaded.load(Ordering::Relaxed);
                    (s.start + downloaded, s.end, s.downloaded.clone())
                })
                .filter(|(start, end, _)| start < end)
                .collect();

            (
                dl.url.clone(),
                dl.save_path.clone(),
                seg_info,
                dl.pause_flag.clone(),
                dl.cancel_flag.clone(),
            )
        };

        let mut new_handles = Vec::new();
        for (start, end, downloaded) in segments_info {
            let already = downloaded.load(Ordering::Relaxed);
            let actual_start = start.min(end);
            if actual_start >= end {
                continue;
            }
            downloaded.store(actual_start - (start - already).min(actual_start), Ordering::Relaxed);

            let handle = tokio::spawn(download_segment(
                self.client.clone(),
                url.clone(),
                save_path.clone(),
                actual_start,
                end,
                downloaded,
                pause_flag.clone(),
                cancel_flag.clone(),
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
        let (url, save_dir) = {
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
            (dl.url.clone(), save_dir)
        };

        self.remove(id);
        self.add_download(url, save_dir).await
    }

    pub fn update_state(&self) {
        let mut downloads = self.state.lock().unwrap();
        for dl in downloads.iter_mut() {
            if dl.status != DownloadStatus::Downloading {
                continue;
            }

            let total_downloaded: u64 = dl
                .segments
                .iter()
                .map(|s| s.downloaded.load(Ordering::Relaxed))
                .sum();

            dl.speed_tracker.record(total_downloaded);

            let finished_count = dl.task_handles.iter().filter(|h| h.is_finished()).count();
            let total_tasks = dl.task_handles.len();

            if finished_count == total_tasks {
                let has_error = dl.total_size.is_some()
                    && dl.segments.iter().any(|s| {
                        if s.end == u64::MAX {
                            return false;
                        }
                        let expected = s.end - s.start;
                        s.downloaded.load(Ordering::Relaxed) < expected
                    });

                if has_error {
                    dl.status = DownloadStatus::Failed;
                    dl.error = Some("Download incomplete".to_string());
                } else {
                    dl.status = DownloadStatus::Completed;
                }
                dl.speed_tracker.reset();
                continue;
            }

            if finished_count > 0 && dl.resumable {
                rebalance_segments(dl, &self.client);
            }
        }
    }

    pub fn get_snapshots(&self) -> Vec<DownloadItem> {
        let downloads = self.state.lock().unwrap();
        downloads
            .iter()
            .map(|dl| {
                let _total_downloaded: u64 = dl
                    .segments
                    .iter()
                    .map(|s| s.downloaded.load(Ordering::Relaxed))
                    .sum();

                build_snapshot(
                    dl.id,
                    &dl.url,
                    &dl.filename,
                    &dl.save_path,
                    dl.total_size,
                    dl.status,
                    &dl.segments,
                    dl.speed_tracker.speed(),
                    dl.category,
                    dl.error.clone(),
                    dl.resumable,
                )
            })
            .collect()
    }

    pub fn total_speed(&self) -> f64 {
        let downloads = self.state.lock().unwrap();
        downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Downloading)
            .map(|d| d.speed_tracker.speed())
            .sum()
    }

    pub fn has_active_downloads(&self) -> bool {
        let downloads = self.state.lock().unwrap();
        downloads.iter().any(|d| d.status.is_active())
    }

    pub fn count_by_status(&self) -> (usize, usize, usize, usize, usize) {
        let downloads = self.state.lock().unwrap();
        let total = downloads.len();
        let active = downloads.iter().filter(|d| d.status.is_active()).count();
        let completed = downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Completed)
            .count();
        let paused = downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Paused)
            .count();
        let failed = downloads
            .iter()
            .filter(|d| {
                matches!(
                    d.status,
                    DownloadStatus::Failed | DownloadStatus::Cancelled
                )
            })
            .count();
        (total, active, completed, paused, failed)
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

    // Scale: <5MB=1, <20MB=2, <50MB=4, <200MB=6, >=200MB=8
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

fn rebalance_segments(dl: &mut ManagedDownload, client: &reqwest::Client) {
    dl.task_handles.retain(|h| !h.is_finished());

    let mut remaining: Vec<(usize, u64)> = dl
        .segments
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if s.end == u64::MAX {
                return None;
            }
            let done = s.downloaded.load(Ordering::Relaxed);
            let left = (s.end - s.start).saturating_sub(done);
            if left > MIN_SEGMENT_SIZE * 2 {
                Some((i, left))
            } else {
                None
            }
        })
        .collect();

    remaining.sort_by(|a, b| b.1.cmp(&a.1));

    let idle_slots = dl
        .segments
        .len()
        .saturating_sub(dl.task_handles.len())
        .min(remaining.len());

    if idle_slots == 0 {
        return;
    }

    let mut new_segments = Vec::new();
    let mut new_handles = Vec::new();

    for &(seg_idx, _remaining_bytes) in remaining.iter().take(idle_slots) {
        let seg = &dl.segments[seg_idx];
        let done = seg.downloaded.load(Ordering::Relaxed);
        let current_pos = seg.start + done;
        let midpoint = current_pos + (seg.end - current_pos) / 2;

        if midpoint >= seg.end || seg.end - midpoint < MIN_SEGMENT_SIZE {
            continue;
        }

        let new_seg = SegmentState {
            start: midpoint,
            end: seg.end,
            downloaded: Arc::new(AtomicU64::new(0)),
        };

        let handle = tokio::spawn(download_segment(
            client.clone(),
            dl.url.clone(),
            dl.save_path.clone(),
            midpoint,
            seg.end,
            new_seg.downloaded.clone(),
            dl.pause_flag.clone(),
            dl.cancel_flag.clone(),
        ));

        new_segments.push(new_seg);
        new_handles.push(handle);
    }

    dl.segments.extend(new_segments);
    dl.task_handles.extend(new_handles);
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
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.last() {
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

fn build_snapshot(
    id: Uuid,
    url: &str,
    filename: &str,
    save_path: &PathBuf,
    total_size: Option<u64>,
    status: DownloadStatus,
    segments: &[SegmentState],
    speed: f64,
    category: FileCategory,
    error: Option<String>,
    resumable: bool,
) -> DownloadItem {
    let total_downloaded: u64 = segments
        .iter()
        .map(|s| s.downloaded.load(Ordering::Relaxed))
        .sum();

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
        save_path: save_path.clone(),
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
