use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
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
