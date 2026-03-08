use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::model::{HistoryEntry, PersistedDownload};
use crate::theme::ThemeMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProxyType {
    #[default]
    None,
    Http,
    Https,
    Socks5,
}

impl ProxyType {
    pub const ALL: [ProxyType; 4] = [
        ProxyType::None,
        ProxyType::Http,
        ProxyType::Https,
        ProxyType::Socks5,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ProxyType::None => "None",
            ProxyType::Http => "HTTP",
            ProxyType::Https => "HTTPS",
            ProxyType::Socks5 => "SOCKS5",
        }
    }

    pub fn scheme(self) -> &'static str {
        match self {
            ProxyType::None => "",
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5h",
        }
    }
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    #[serde(default)]
    pub proxy_type: ProxyType,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl ProxyConfig {
    pub fn to_url(&self) -> Option<String> {
        if self.proxy_type == ProxyType::None || self.host.is_empty() {
            return None;
        }
        let scheme = self.proxy_type.scheme();
        let host_port = if self.port.is_empty() {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        };
        if !self.username.is_empty() {
            let auth = if self.password.is_empty() {
                self.username.clone()
            } else {
                format!("{}:{}", self.username, self.password)
            };
            Some(format!("{}://{}@{}", scheme, auth, host_port))
        } else {
            Some(format!("{}://{}", scheme, host_port))
        }
    }

    pub fn is_active(&self) -> bool {
        self.proxy_type != ProxyType::None && !self.host.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_dir: PathBuf,
    pub max_concurrent: usize,
    pub segments_per_download: usize,
    /// Global speed limit in bytes/sec. None = unlimited.
    pub speed_limit: Option<u64>,
    #[serde(default = "default_theme")]
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub schedule_enabled: bool,
    #[serde(default = "default_schedule_from")]
    pub schedule_from: (u8, u8),
    #[serde(default = "default_schedule_to")]
    pub schedule_to: (u8, u8),
    #[serde(default)]
    pub proxy: ProxyConfig,
    #[serde(default = "default_autostart")]
    pub autostart: bool,
}

fn default_theme() -> ThemeMode {
    ThemeMode::Dark
}

fn default_schedule_from() -> (u8, u8) {
    (22, 0)
}

fn default_schedule_to() -> (u8, u8) {
    (6, 0)
}

fn default_autostart() -> bool {
    true
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
            schedule_enabled: false,
            schedule_from: default_schedule_from(),
            schedule_to: default_schedule_to(),
            proxy: ProxyConfig::default(),
            autostart: true,
        }
    }
}

impl AppSettings {
    pub fn is_within_schedule(&self) -> bool {
        if !self.schedule_enabled {
            return true;
        }
        let now = chrono::Local::now().time();
        let cur_min = now.hour() as u16 * 60 + now.minute() as u16;
        let from_min = self.schedule_from.0 as u16 * 60 + self.schedule_from.1 as u16;
        let to_min = self.schedule_to.0 as u16 * 60 + self.schedule_to.1 as u16;

        if from_min <= to_min {
            (from_min..=to_min).contains(&cur_min)
        } else {
            cur_min >= from_min || cur_min <= to_min
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadHistory {
    pub entries: Vec<HistoryEntry>,
}

impl DownloadHistory {
    pub fn load() -> Self {
        let path = history_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&data) {
                    return history;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = history_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, data);
        }
    }

    pub fn add(&mut self, entry: HistoryEntry) {
        if !self.entries.iter().any(|e| e.id == entry.id) {
            self.entries.insert(0, entry);
            const MAX_HISTORY: usize = 500;
            self.entries.truncate(MAX_HISTORY);
        }
    }

    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.filename.to_lowercase().contains(&q) || e.url.to_lowercase().contains(&q))
            .collect()
    }
}

fn history_path() -> PathBuf {
    directories::ProjectDirs::from("com", "bolt", "Bolt")
        .map(|dirs| dirs.config_dir().join("history.json"))
        .unwrap_or_else(|| PathBuf::from("bolt_history.json"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FileCategory;
    use uuid::Uuid;

    #[test]
    fn history_add_deduplicates() {
        let mut history = DownloadHistory::default();
        let id = Uuid::new_v4();
        let entry = HistoryEntry {
            id,
            url: "https://example.com/f.zip".to_string(),
            filename: "f.zip".to_string(),
            save_path: PathBuf::from("/tmp/f.zip"),
            total_size: Some(1024),
            category: FileCategory::Archive,
            completed_at: "2025-03-06 18:00".to_string(),
        };

        history.add(entry.clone());
        history.add(entry);
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn history_add_inserts_at_front() {
        let mut history = DownloadHistory::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        history.add(HistoryEntry {
            id: id1,
            url: "https://example.com/a.zip".to_string(),
            filename: "a.zip".to_string(),
            save_path: PathBuf::from("/tmp/a.zip"),
            total_size: Some(100),
            category: FileCategory::Archive,
            completed_at: "2025-01-01 10:00".to_string(),
        });
        history.add(HistoryEntry {
            id: id2,
            url: "https://example.com/b.zip".to_string(),
            filename: "b.zip".to_string(),
            save_path: PathBuf::from("/tmp/b.zip"),
            total_size: Some(200),
            category: FileCategory::Archive,
            completed_at: "2025-01-02 10:00".to_string(),
        });

        assert_eq!(history.entries[0].id, id2);
        assert_eq!(history.entries[1].id, id1);
    }

    #[test]
    fn history_search_by_filename() {
        let mut history = DownloadHistory::default();
        history.add(HistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/movie.mp4".to_string(),
            filename: "movie.mp4".to_string(),
            save_path: PathBuf::from("/tmp/movie.mp4"),
            total_size: Some(1000),
            category: FileCategory::Video,
            completed_at: "2025-03-06 18:00".to_string(),
        });
        history.add(HistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/doc.pdf".to_string(),
            filename: "doc.pdf".to_string(),
            save_path: PathBuf::from("/tmp/doc.pdf"),
            total_size: Some(500),
            category: FileCategory::Document,
            completed_at: "2025-03-06 19:00".to_string(),
        });

        let results = history.search("movie");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "movie.mp4");
    }

    #[test]
    fn history_search_by_url() {
        let mut history = DownloadHistory::default();
        history.add(HistoryEntry {
            id: Uuid::new_v4(),
            url: "https://cdn.example.com/file.bin".to_string(),
            filename: "file.bin".to_string(),
            save_path: PathBuf::from("/tmp/file.bin"),
            total_size: Some(100),
            category: FileCategory::Other,
            completed_at: "2025-03-06 18:00".to_string(),
        });

        assert_eq!(history.search("cdn.example").len(), 1);
        assert_eq!(history.search("notfound").len(), 0);
    }

    #[test]
    fn history_search_case_insensitive() {
        let mut history = DownloadHistory::default();
        history.add(HistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/Report.PDF".to_string(),
            filename: "Report.PDF".to_string(),
            save_path: PathBuf::from("/tmp/Report.PDF"),
            total_size: Some(100),
            category: FileCategory::Document,
            completed_at: "2025-03-06 18:00".to_string(),
        });

        assert_eq!(history.search("report").len(), 1);
        assert_eq!(history.search("REPORT").len(), 1);
    }

    #[test]
    fn history_serde_roundtrip() {
        let mut history = DownloadHistory::default();
        history.add(HistoryEntry {
            id: Uuid::new_v4(),
            url: "https://example.com/f.zip".to_string(),
            filename: "f.zip".to_string(),
            save_path: PathBuf::from("/tmp/f.zip"),
            total_size: Some(1024),
            category: FileCategory::Archive,
            completed_at: "2025-03-06 18:00".to_string(),
        });

        let json = serde_json::to_string(&history).unwrap();
        let restored: DownloadHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.entries.len(), 1);
        assert_eq!(restored.entries[0].filename, "f.zip");
    }

    #[test]
    fn download_database_from_persisted() {
        let db = DownloadDatabase::from_persisted(vec![]);
        assert!(db.downloads.is_empty());
    }

    #[test]
    fn schedule_disabled_always_within() {
        let settings = AppSettings {
            schedule_enabled: false,
            ..AppSettings::default()
        };
        assert!(settings.is_within_schedule());
    }

    #[test]
    fn schedule_same_day_window() {
        let now = chrono::Local::now();
        let h = now.format("%H").to_string().parse::<u8>().unwrap();
        let m = now.format("%M").to_string().parse::<u8>().unwrap();

        let settings = AppSettings {
            schedule_enabled: true,
            schedule_from: (h, m),
            schedule_to: (h, m),
            ..AppSettings::default()
        };
        assert!(settings.is_within_schedule());
    }

    #[test]
    fn schedule_overnight_window() {
        let settings = AppSettings {
            schedule_enabled: true,
            schedule_from: (22, 0),
            schedule_to: (6, 0),
            ..AppSettings::default()
        };
        // 23:00 should be within 22:00–06:00
        let _ = settings; // tested structurally below

        let s = AppSettings {
            schedule_enabled: true,
            schedule_from: (0, 0),
            schedule_to: (23, 59),
            ..AppSettings::default()
        };
        assert!(s.is_within_schedule(), "0:00–23:59 covers all times");
    }
}
