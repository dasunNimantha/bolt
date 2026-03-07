use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::model::PersistedDownload;
use crate::theme::ThemeMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_dir: PathBuf,
    pub max_concurrent: usize,
    pub segments_per_download: usize,
    /// Global speed limit in bytes/sec. None = unlimited.
    pub speed_limit: Option<u64>,
    #[serde(default = "default_theme")]
    pub theme_mode: ThemeMode,
}

fn default_theme() -> ThemeMode {
    ThemeMode::Dark
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs_default_download();
        Self {
            download_dir,
            max_concurrent: 3,
            segments_per_download: 8,
            speed_limit: None,
            theme_mode: ThemeMode::Dark,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&data) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, data);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadDatabase {
    pub downloads: Vec<PersistedDownload>,
}

impl DownloadDatabase {
    pub fn load() -> Self {
        let path = downloads_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(db) = serde_json::from_str(&data) {
                    return db;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = downloads_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, data);
        }
    }

    pub fn from_persisted(downloads: Vec<PersistedDownload>) -> Self {
        Self { downloads }
    }
}

fn config_path() -> PathBuf {
    directories::ProjectDirs::from("com", "bolt", "Bolt")
        .map(|dirs| dirs.config_dir().join("settings.json"))
        .unwrap_or_else(|| PathBuf::from("bolt_settings.json"))
}

fn downloads_path() -> PathBuf {
    directories::ProjectDirs::from("com", "bolt", "Bolt")
        .map(|dirs| dirs.config_dir().join("downloads.json"))
        .unwrap_or_else(|| PathBuf::from("bolt_downloads.json"))
}

fn dirs_default_download() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.download_dir().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Downloads")
        })
}

mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}
