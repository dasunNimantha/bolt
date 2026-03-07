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

    // Settings
    ChooseDownloadDir,
    DownloadDirChosen(Option<std::path::PathBuf>),
    SetMaxConcurrent(String),
    SetSpeedLimit(String),
    ClearSpeedLimit,

    // Scheduling
    ScheduleDownload(Uuid, String),
    ClearSchedule(Uuid),

    // Tray / window
    WindowCloseRequested,
    TrayShow,
    TrayQuit,

    Noop,
}
