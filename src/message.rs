use crate::model::{DownloadFilter, DownloadItem};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Message {
    // URL input
    UrlInputChanged(String),
    AddDownload,
    DownloadAdded(Box<DownloadItem>),
    DownloadError(String),

    // Download actions
    StartDownload(Uuid),
    PauseDownload(Uuid),
    ResumeDownload(Uuid),
    CancelDownload(Uuid),
    RemoveDownload(Uuid),
    RetryDownload(Uuid),
    ClearCompleted,

    // Navigation
    SelectDownload(Option<Uuid>),
    FilterChanged(DownloadFilter),

    // UI
    ToggleTheme,
    Tick,
    OpenFile(Uuid),
    OpenFolder(Uuid),
    ShowSettings,
    ShowDownloads,

    // Search
    SearchChanged(String),

    // Settings
    ChooseDownloadDir,
    DownloadDirChosen(Option<std::path::PathBuf>),
    SetMaxConcurrent(String),
    SetSpeedLimit(String),
    ClearSpeedLimit,

    // Schedule window
    ToggleSchedule,
    SetScheduleFromH(String),
    SetScheduleFromM(String),
    SetScheduleToH(String),
    SetScheduleToM(String),

    // Network / auto-resume
    NetworkStatus(bool),

    // Tray / window
    WindowCloseRequested,
    TrayShow,
    TrayQuit,

    Noop,
}
