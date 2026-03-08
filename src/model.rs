use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Connecting => "Connecting",
            Self::Downloading => "Downloading",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Connecting | Self::Downloading)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    Video,
    Audio,
    Document,
    Archive,
    Image,
    Application,
    Other,
}

impl FileCategory {
    pub fn from_filename(filename: &str) -> Self {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

        match ext.as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
            | "3gp" | "ts" => Self::Video,
            "mp3" | "flac" | "wav" | "aac" | "ogg" | "wma" | "m4a" | "opus" => Self::Audio,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "rtf" | "odt"
            | "csv" | "epub" => Self::Document,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" | "iso" | "dmg" => {
                Self::Archive
            }
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif" => {
                Self::Image
            }
            "exe" | "msi" | "deb" | "rpm" | "appimage" | "flatpak" | "snap" | "apk" | "bin"
            | "run" | "sh" => Self::Application,
            _ => Self::Other,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Document => "Document",
            Self::Archive => "Archive",
            Self::Image => "Image",
            Self::Application => "Application",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadFilter {
    #[default]
    All,
    Active,
    Completed,
    Paused,
    Failed,
}

impl DownloadFilter {
    pub fn matches(&self, status: DownloadStatus) -> bool {
        match self {
            Self::All => true,
            Self::Active => status.is_active(),
            Self::Completed => status == DownloadStatus::Completed,
            Self::Paused => status == DownloadStatus::Paused,
            Self::Failed => {
                matches!(status, DownloadStatus::Failed | DownloadStatus::Cancelled)
            }
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
            Self::Paused => "Paused",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Downloads,
    Settings,
}

#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSegment {
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedDownload {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub save_path: PathBuf,
    pub total_size: Option<u64>,
    pub status: DownloadStatus,
    pub segments: Vec<PersistedSegment>,
    pub category: FileCategory,
    pub error: Option<String>,
    pub resumable: bool,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedFileInfo {
    pub filename: String,
    pub total_size: Option<u64>,
    pub resumable: bool,
}

#[derive(Debug, Clone)]
pub struct PendingDownload {
    pub url: String,
    pub filename: Option<String>,
    pub headers: HashMap<String, String>,
    pub resolved: Option<ResolvedFileInfo>,
}

impl PendingDownload {
    pub fn display_filename(&self) -> String {
        if let Some(ref info) = self.resolved {
            return info.filename.clone();
        }
        if let Some(ref name) = self.filename {
            if !name.is_empty() {
                return name.clone();
            }
        }
        let path = self.url.rsplit('/').next().unwrap_or("download");
        let path = path.split('?').next().unwrap_or(path);
        let path = path.split('#').next().unwrap_or(path);
        if path.is_empty() {
            "download".to_string()
        } else {
            path.to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub save_path: PathBuf,
    pub total_size: Option<u64>,
    pub category: FileCategory,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub save_path: PathBuf,
    pub total_size: Option<u64>,
    pub downloaded: u64,
    pub status: DownloadStatus,
    pub segments: Vec<SegmentInfo>,
    pub speed: f64,
    pub category: FileCategory,
    pub error: Option<String>,
    pub resumable: bool,
}

impl DownloadItem {
    pub fn progress_percent(&self) -> f32 {
        match self.total_size {
            Some(total) if total > 0 => (self.downloaded as f64 / total as f64 * 100.0) as f32,
            _ => 0.0,
        }
    }

    pub fn eta_seconds(&self) -> Option<u64> {
        if self.speed <= 0.0 {
            return None;
        }
        let remaining = self.total_size.unwrap_or(0).saturating_sub(self.downloaded);
        Some((remaining as f64 / self.speed) as u64)
    }
}

#[derive(Debug, Clone)]
pub struct SpeedTracker {
    samples: VecDeque<(Instant, u64)>,
    max_samples: usize,
}

impl Default for SpeedTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeedTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            max_samples: 8,
        }
    }

    pub fn record(&mut self, downloaded: u64) {
        let now = Instant::now();
        if let Some(last) = self.samples.back() {
            if now.duration_since(last.0).as_millis() < 100 {
                return;
            }
        }
        self.samples.push_back((now, downloaded));
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    pub fn speed(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let oldest = self.samples.front().unwrap();
        let newest = self.samples.back().unwrap();
        let duration = newest.0.duration_since(oldest.0).as_secs_f64();
        if duration < 0.1 {
            return 0.0;
        }
        (newest.1.saturating_sub(oldest.1)) as f64 / duration
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_status_labels() {
        assert_eq!(DownloadStatus::Queued.label(), "Queued");
        assert_eq!(DownloadStatus::Downloading.label(), "Downloading");
        assert_eq!(DownloadStatus::Paused.label(), "Paused");
        assert_eq!(DownloadStatus::Completed.label(), "Completed");
        assert_eq!(DownloadStatus::Failed.label(), "Failed");
        assert_eq!(DownloadStatus::Cancelled.label(), "Cancelled");
        assert_eq!(DownloadStatus::Connecting.label(), "Connecting");
    }

    #[test]
    fn download_status_is_active() {
        assert!(DownloadStatus::Queued.is_active());
        assert!(DownloadStatus::Connecting.is_active());
        assert!(DownloadStatus::Downloading.is_active());
        assert!(!DownloadStatus::Paused.is_active());
        assert!(!DownloadStatus::Completed.is_active());
        assert!(!DownloadStatus::Failed.is_active());
        assert!(!DownloadStatus::Cancelled.is_active());
    }

    #[test]
    fn file_category_detection() {
        assert_eq!(
            FileCategory::from_filename("movie.mp4"),
            FileCategory::Video
        );
        assert_eq!(
            FileCategory::from_filename("movie.MKV"),
            FileCategory::Video
        );
        assert_eq!(FileCategory::from_filename("song.mp3"), FileCategory::Audio);
        assert_eq!(
            FileCategory::from_filename("song.flac"),
            FileCategory::Audio
        );
        assert_eq!(
            FileCategory::from_filename("doc.pdf"),
            FileCategory::Document
        );
        assert_eq!(
            FileCategory::from_filename("doc.xlsx"),
            FileCategory::Document
        );
        assert_eq!(
            FileCategory::from_filename("archive.zip"),
            FileCategory::Archive
        );
        assert_eq!(
            FileCategory::from_filename("archive.tar"),
            FileCategory::Archive
        );
        assert_eq!(
            FileCategory::from_filename("photo.jpg"),
            FileCategory::Image
        );
        assert_eq!(
            FileCategory::from_filename("photo.PNG"),
            FileCategory::Image
        );
        assert_eq!(
            FileCategory::from_filename("app.deb"),
            FileCategory::Application
        );
        assert_eq!(
            FileCategory::from_filename("app.exe"),
            FileCategory::Application
        );
        assert_eq!(FileCategory::from_filename("readme"), FileCategory::Other);
        assert_eq!(FileCategory::from_filename("data.xyz"), FileCategory::Other);
    }

    #[test]
    fn file_category_labels() {
        assert_eq!(FileCategory::Video.label(), "Video");
        assert_eq!(FileCategory::Audio.label(), "Audio");
        assert_eq!(FileCategory::Document.label(), "Document");
        assert_eq!(FileCategory::Archive.label(), "Archive");
        assert_eq!(FileCategory::Image.label(), "Image");
        assert_eq!(FileCategory::Application.label(), "Application");
        assert_eq!(FileCategory::Other.label(), "Other");
    }

    #[test]
    fn download_filter_matches() {
        assert!(DownloadFilter::All.matches(DownloadStatus::Downloading));
        assert!(DownloadFilter::All.matches(DownloadStatus::Completed));
        assert!(DownloadFilter::All.matches(DownloadStatus::Failed));

        assert!(DownloadFilter::Active.matches(DownloadStatus::Downloading));
        assert!(DownloadFilter::Active.matches(DownloadStatus::Queued));
        assert!(!DownloadFilter::Active.matches(DownloadStatus::Paused));

        assert!(DownloadFilter::Completed.matches(DownloadStatus::Completed));
        assert!(!DownloadFilter::Completed.matches(DownloadStatus::Downloading));

        assert!(DownloadFilter::Paused.matches(DownloadStatus::Paused));
        assert!(!DownloadFilter::Paused.matches(DownloadStatus::Downloading));

        assert!(DownloadFilter::Failed.matches(DownloadStatus::Failed));
        assert!(DownloadFilter::Failed.matches(DownloadStatus::Cancelled));
        assert!(!DownloadFilter::Failed.matches(DownloadStatus::Completed));
    }

    #[test]
    fn progress_percent_with_size() {
        let item = DownloadItem {
            id: Uuid::new_v4(),
            url: String::new(),
            filename: String::new(),
            save_path: PathBuf::new(),
            total_size: Some(1000),
            downloaded: 500,
            status: DownloadStatus::Downloading,
            segments: vec![],
            speed: 0.0,
            category: FileCategory::Other,
            error: None,
            resumable: false,
        };
        assert!((item.progress_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn progress_percent_no_size() {
        let item = DownloadItem {
            id: Uuid::new_v4(),
            url: String::new(),
            filename: String::new(),
            save_path: PathBuf::new(),
            total_size: None,
            downloaded: 500,
            status: DownloadStatus::Downloading,
            segments: vec![],
            speed: 0.0,
            category: FileCategory::Other,
            error: None,
            resumable: false,
        };
        assert!((item.progress_percent() - 0.0).abs() < 0.01);
    }

    #[test]
    fn progress_percent_zero_size() {
        let item = DownloadItem {
            id: Uuid::new_v4(),
            url: String::new(),
            filename: String::new(),
            save_path: PathBuf::new(),
            total_size: Some(0),
            downloaded: 0,
            status: DownloadStatus::Downloading,
            segments: vec![],
            speed: 0.0,
            category: FileCategory::Other,
            error: None,
            resumable: false,
        };
        assert!((item.progress_percent() - 0.0).abs() < 0.01);
    }

    #[test]
    fn eta_with_speed() {
        let item = DownloadItem {
            id: Uuid::new_v4(),
            url: String::new(),
            filename: String::new(),
            save_path: PathBuf::new(),
            total_size: Some(10000),
            downloaded: 5000,
            status: DownloadStatus::Downloading,
            segments: vec![],
            speed: 1000.0,
            category: FileCategory::Other,
            error: None,
            resumable: false,
        };
        assert_eq!(item.eta_seconds(), Some(5));
    }

    #[test]
    fn eta_without_speed() {
        let item = DownloadItem {
            id: Uuid::new_v4(),
            url: String::new(),
            filename: String::new(),
            save_path: PathBuf::new(),
            total_size: Some(10000),
            downloaded: 5000,
            status: DownloadStatus::Downloading,
            segments: vec![],
            speed: 0.0,
            category: FileCategory::Other,
            error: None,
            resumable: false,
        };
        assert_eq!(item.eta_seconds(), None);
    }

    #[test]
    fn speed_tracker_empty() {
        let tracker = SpeedTracker::new();
        assert!((tracker.speed() - 0.0).abs() < 0.01);
    }

    #[test]
    fn speed_tracker_single_sample() {
        let mut tracker = SpeedTracker::new();
        tracker.samples.push_back((Instant::now(), 100));
        assert!((tracker.speed() - 0.0).abs() < 0.01);
    }

    #[test]
    fn speed_tracker_reset() {
        let mut tracker = SpeedTracker::new();
        tracker.samples.push_back((Instant::now(), 100));
        tracker.reset();
        assert!(tracker.samples.is_empty());
    }

    #[test]
    fn speed_tracker_default() {
        let tracker = SpeedTracker::default();
        assert!(tracker.samples.is_empty());
        assert_eq!(tracker.max_samples, 8);
    }

    #[test]
    fn persisted_download_serde_roundtrip() {
        let pd = PersistedDownload {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            save_path: PathBuf::from("/tmp/file.zip"),
            total_size: Some(1024),
            status: DownloadStatus::Paused,
            segments: vec![
                PersistedSegment {
                    start: 0,
                    end: 512,
                    downloaded: 256,
                },
                PersistedSegment {
                    start: 512,
                    end: 1024,
                    downloaded: 100,
                },
            ],
            category: FileCategory::Archive,
            error: None,
            resumable: true,
            headers: HashMap::new(),
        };

        let json = serde_json::to_string(&pd).unwrap();
        let restored: PersistedDownload = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, pd.id);
        assert_eq!(restored.url, pd.url);
        assert_eq!(restored.filename, pd.filename);
        assert_eq!(restored.total_size, pd.total_size);
        assert_eq!(restored.segments.len(), 2);
        assert_eq!(restored.segments[0].downloaded, 256);
    }

    #[test]
    fn view_mode_equality() {
        assert_eq!(ViewMode::Downloads, ViewMode::Downloads);
        assert_eq!(ViewMode::Settings, ViewMode::Settings);
        assert_ne!(ViewMode::Downloads, ViewMode::Settings);
    }

    #[test]
    fn history_entry_serde_roundtrip() {
        let entry = HistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            save_path: PathBuf::from("/tmp/file.zip"),
            total_size: Some(1024),
            category: FileCategory::Archive,
            completed_at: "2025-03-06 18:00".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let restored: HistoryEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, entry.id);
        assert_eq!(restored.filename, "file.zip");
        assert_eq!(restored.completed_at, "2025-03-06 18:00");
    }

    #[test]
    fn display_filename_with_resolved() {
        let pd = PendingDownload {
            url: "https://example.com/other.bin".to_string(),
            filename: Some("explicit.zip".to_string()),
            headers: HashMap::new(),
            resolved: Some(ResolvedFileInfo {
                filename: "resolved.tar.gz".to_string(),
                total_size: Some(1024),
                resumable: true,
            }),
        };
        assert_eq!(pd.display_filename(), "resolved.tar.gz");
    }

    #[test]
    fn display_filename_with_explicit() {
        let pd = PendingDownload {
            url: "https://example.com/other.bin".to_string(),
            filename: Some("explicit.zip".to_string()),
            headers: HashMap::new(),
            resolved: None,
        };
        assert_eq!(pd.display_filename(), "explicit.zip");
    }

    #[test]
    fn display_filename_from_url() {
        let pd = PendingDownload {
            url: "https://example.com/downloads/archive.tar.gz".to_string(),
            filename: None,
            headers: HashMap::new(),
            resolved: None,
        };
        assert_eq!(pd.display_filename(), "archive.tar.gz");
    }

    #[test]
    fn display_filename_url_with_query() {
        let pd = PendingDownload {
            url: "https://example.com/file.zip?token=abc&v=2".to_string(),
            filename: None,
            headers: HashMap::new(),
            resolved: None,
        };
        assert_eq!(pd.display_filename(), "file.zip");
    }

    #[test]
    fn display_filename_url_with_fragment() {
        let pd = PendingDownload {
            url: "https://example.com/file.zip#section".to_string(),
            filename: None,
            headers: HashMap::new(),
            resolved: None,
        };
        assert_eq!(pd.display_filename(), "file.zip");
    }

    #[test]
    fn display_filename_empty_explicit_falls_through() {
        let pd = PendingDownload {
            url: "https://example.com/fallback.bin".to_string(),
            filename: Some(String::new()),
            headers: HashMap::new(),
            resolved: None,
        };
        assert_eq!(pd.display_filename(), "fallback.bin");
    }

    #[test]
    fn display_filename_trailing_slash() {
        let pd = PendingDownload {
            url: "https://example.com/path/".to_string(),
            filename: None,
            headers: HashMap::new(),
            resolved: None,
        };
        assert_eq!(pd.display_filename(), "download");
    }

    #[test]
    fn download_filter_labels() {
        assert_eq!(DownloadFilter::All.label(), "All");
        assert_eq!(DownloadFilter::Active.label(), "Active");
        assert_eq!(DownloadFilter::Completed.label(), "Completed");
        assert_eq!(DownloadFilter::Paused.label(), "Paused");
        assert_eq!(DownloadFilter::Failed.label(), "Failed");
    }
}
