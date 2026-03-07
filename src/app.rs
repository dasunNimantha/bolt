use crate::download::engine::DownloadEngine;
use crate::message::Message;
use crate::model::{DownloadFilter, DownloadItem, DownloadStatus, HistoryEntry, ViewMode};
use crate::settings::{AppSettings, DownloadDatabase, DownloadHistory};
use crate::theme::{bolt_theme, ThemeMode};
use crate::tray::BoltTray;
use crate::utils::format::format_speed;
use crate::view::build_view;
use chrono::Local;
use iced::{event, window, Application, Command, Element, Event, Subscription, Theme};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

const NETWORK_CHECK_INTERVAL: u32 = 30;

pub struct BoltApp {
    engine: Arc<DownloadEngine>,
    downloads: Vec<DownloadItem>,
    selected: Option<Uuid>,
    url_input: String,
    search_query: String,
    filter: DownloadFilter,
    settings: AppSettings,
    total_speed: f64,
    counts: (usize, usize, usize, usize, usize),
    error_message: Option<String>,
    adding: bool,
    view_mode: ViewMode,
    speed_limit_input: String,
    max_concurrent_input: String,
    sched_from_h: String,
    sched_from_m: String,
    sched_to_h: String,
    sched_to_m: String,
    persist_counter: u32,
    tray: Option<BoltTray>,
    network_online: bool,
    network_check_counter: u32,
    history: DownloadHistory,
    /// Tracks whether we were inside the schedule window on the previous check,
    /// so we only trigger auto-start on the outside→inside transition.
    schedule_was_active: bool,
}

impl Application for BoltApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let settings = AppSettings::load();
        let engine = Arc::new(DownloadEngine::new());

        if let Some(limit) = settings.speed_limit {
            engine.set_speed_limit(limit);
        }
        engine.set_max_concurrent(settings.max_concurrent as u64);

        let db = DownloadDatabase::load();
        engine.restore_downloads(&db);

        let speed_limit_input = match settings.speed_limit {
            Some(bps) => format!("{}", bps / 1024),
            None => String::new(),
        };
        let max_concurrent_input = format!("{}", settings.max_concurrent);
        let sched_from_h = format!("{:02}", settings.schedule_from.0);
        let sched_from_m = format!("{:02}", settings.schedule_from.1);
        let sched_to_h = format!("{:02}", settings.schedule_to.0);
        let sched_to_m = format!("{:02}", settings.schedule_to.1);

        let tray = BoltTray::new();
        let history = DownloadHistory::load();
        let schedule_was_active =
            settings.schedule_enabled && settings.is_within_schedule();

        let mut app = Self {
            engine,
            downloads: Vec::new(),
            selected: None,
            url_input: String::new(),
            search_query: String::new(),
            filter: DownloadFilter::All,
            settings,
            total_speed: 0.0,
            counts: (0, 0, 0, 0, 0),
            error_message: None,
            adding: false,
            view_mode: ViewMode::Downloads,
            speed_limit_input,
            max_concurrent_input,
            sched_from_h,
            sched_from_m,
            sched_to_h,
            sched_to_m,
            persist_counter: 0,
            tray,
            network_online: true,
            network_check_counter: 0,
            history,
            schedule_was_active,
        };
        app.refresh_snapshots();

        (app, Command::none())
    }

    fn title(&self) -> String {
        let (total, active, ..) = self.counts;
        if active > 0 {
            format!("Bolt - {} active / {} total", active, total)
        } else if total > 0 {
            format!("Bolt - {} downloads", total)
        } else {
            "Bolt - Download Manager".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UrlInputChanged(url) => {
                self.url_input = url;
                self.error_message = None;
                Command::none()
            }

            Message::AddDownload => {
                let mut url = self.url_input.trim().to_string();
                if url.is_empty() {
                    return Command::none();
                }

                if !url.starts_with("http://") && !url.starts_with("https://") {
                    url = format!("https://{}", url);
                }

                if url::Url::parse(&url).is_err() {
                    self.error_message = Some("Invalid URL".to_string());
                    return Command::none();
                }

                self.url_input.clear();
                self.adding = true;

                let engine = self.engine.clone();
                let save_dir = self.settings.download_dir.clone();

                Command::perform(
                    async move {
                        match engine.add_download(url, save_dir).await {
                            Ok(item) => Message::DownloadAdded(Box::new(item)),
                            Err(e) => Message::DownloadError(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }

            Message::DownloadAdded(_item) => {
                self.adding = false;
                self.error_message = None;
                self.refresh_snapshots();
                self.save_downloads();
                Command::none()
            }

            Message::DownloadError(err) => {
                self.adding = false;
                eprintln!("Download error: {}", err);
                self.error_message = Some(err);
                Command::none()
            }

            Message::StartDownload(id) => {
                let engine = self.engine.clone();
                Command::perform(
                    async move {
                        if let Err(e) = engine.start_download(id).await {
                            return Message::DownloadError(e.to_string());
                        }
                        Message::Tick
                    },
                    |msg| msg,
                )
            }

            Message::PauseDownload(id) => {
                self.engine.pause(id);
                self.refresh_snapshots();
                self.save_downloads();
                Command::none()
            }

            Message::ResumeDownload(id) => {
                let engine = self.engine.clone();
                Command::perform(
                    async move {
                        if let Err(e) = engine.resume(id).await {
                            return Message::DownloadError(e.to_string());
                        }
                        Message::Tick
                    },
                    |msg| msg,
                )
            }

            Message::CancelDownload(id) => {
                self.engine.cancel(id);
                self.refresh_snapshots();
                self.save_downloads();
                Command::none()
            }

            Message::RemoveDownload(id) => {
                self.engine.remove(id);
                if self.selected == Some(id) {
                    self.selected = None;
                }
                self.refresh_snapshots();
                self.save_downloads();
                Command::none()
            }

            Message::RetryDownload(id) => {
                let engine = self.engine.clone();
                Command::perform(
                    async move {
                        match engine.retry(id).await {
                            Ok(item) => Message::DownloadAdded(Box::new(item)),
                            Err(e) => Message::DownloadError(e.to_string()),
                        }
                    },
                    |msg| msg,
                )
            }

            Message::ClearCompleted => {
                self.record_completed_to_history();
                self.engine.clear_completed();
                self.refresh_snapshots();
                self.save_downloads();
                self.history.save();
                Command::none()
            }

            Message::SelectDownload(id) => {
                self.selected = id;
                Command::none()
            }

            Message::FilterChanged(filter) => {
                self.filter = filter;
                Command::none()
            }

            Message::ToggleTheme => {
                self.settings.theme_mode = match self.settings.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                self.settings.save();
                Command::none()
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
                Command::none()
            }

            Message::Tick => {
                self.engine.update_state();

                // Check for newly completed downloads and add to history
                let prev_downloads = self.downloads.clone();
                self.refresh_snapshots();
                self.check_newly_completed(&prev_downloads);

                self.update_tray_tooltip();

                self.persist_counter += 1;
                if self.persist_counter.is_multiple_of(8) {
                    self.save_downloads();

                    // Scheduled auto-start: trigger once when the window opens
                    let in_window = self.settings.schedule_enabled
                        && self.settings.is_within_schedule();
                    if in_window && !self.schedule_was_active {
                        let queued = self.engine.get_queued_ids();
                        if !queued.is_empty() {
                            self.schedule_was_active = true;
                            let engine = self.engine.clone();
                            return Command::perform(
                                async move {
                                    for id in queued {
                                        let _ = engine.start_download(id).await;
                                    }
                                    Message::Tick
                                },
                                |msg| msg,
                            );
                        }
                    }
                    self.schedule_was_active = in_window;

                    // Concurrency-blocked downloads: auto-start when a slot frees up
                    let auto_start = self.engine.auto_start_queued();
                    if !auto_start.is_empty() {
                        let engine = self.engine.clone();
                        return Command::perform(
                            async move {
                                for id in auto_start {
                                    let _ = engine.start_download(id).await;
                                }
                                Message::Tick
                            },
                            |msg| msg,
                        );
                    }
                }

                // Network connectivity check
                self.network_check_counter += 1;
                if self.network_check_counter >= NETWORK_CHECK_INTERVAL {
                    self.network_check_counter = 0;
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(5))
                        .build()
                        .unwrap_or_default();
                    return Command::perform(
                        async move {
                            let ok = client
                                .head("https://clients3.google.com/generate_204")
                                .send()
                                .await
                                .is_ok();
                            Message::NetworkStatus(ok)
                        },
                        |msg| msg,
                    );
                }

                if let Some(ref tray) = self.tray {
                    if let Some(action) = tray.poll() {
                        return match action {
                            crate::tray::TrayAction::Show => Command::batch([
                                window::change_mode(window::Id::MAIN, window::Mode::Windowed),
                                window::gain_focus(window::Id::MAIN),
                            ]),
                            crate::tray::TrayAction::Quit => {
                                self.save_downloads();
                                window::close(window::Id::MAIN)
                            }
                        };
                    }
                }

                Command::none()
            }

            Message::NetworkStatus(online) => {
                let was_offline = !self.network_online;
                self.network_online = online;

                if online && was_offline {
                    let failed_ids = self.engine.get_failed_ids();
                    if !failed_ids.is_empty() {
                        let engine = self.engine.clone();
                        return Command::perform(
                            async move {
                                for id in failed_ids {
                                    let _ = engine.retry(id).await;
                                }
                                Message::Tick
                            },
                            |msg| msg,
                        );
                    }
                }

                Command::none()
            }

            Message::OpenFile(id) => {
                if let Some(dl) = self.downloads.iter().find(|d| d.id == id) {
                    let _ = open_path(&dl.save_path);
                }
                Command::none()
            }

            Message::OpenFolder(id) => {
                if let Some(dl) = self.downloads.iter().find(|d| d.id == id) {
                    if let Some(parent) = dl.save_path.parent() {
                        let _ = open_path(parent);
                    }
                }
                Command::none()
            }

            Message::ShowSettings => {
                self.view_mode = ViewMode::Settings;
                Command::none()
            }

            Message::ShowDownloads => {
                self.view_mode = ViewMode::Downloads;
                Command::none()
            }

            Message::ChooseDownloadDir => Command::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Choose Download Directory")
                        .pick_folder()
                        .await;
                    Message::DownloadDirChosen(handle.map(|h| h.path().to_path_buf()))
                },
                |msg| msg,
            ),

            Message::DownloadDirChosen(path) => {
                if let Some(dir) = path {
                    self.settings.download_dir = dir;
                    self.settings.save();
                }
                Command::none()
            }

            Message::SetMaxConcurrent(val) => {
                self.max_concurrent_input = val.clone();
                if let Ok(n) = val.parse::<usize>() {
                    let n = n.clamp(1, 10);
                    self.settings.max_concurrent = n;
                    self.engine.set_max_concurrent(n as u64);
                    self.settings.save();
                }
                Command::none()
            }

            Message::SetSpeedLimit(val) => {
                self.speed_limit_input = val.clone();
                if val.is_empty() {
                    self.settings.speed_limit = None;
                    self.engine.set_speed_limit(0);
                    self.settings.save();
                } else if let Ok(kb) = val.parse::<u64>() {
                    let bps = kb * 1024;
                    self.settings.speed_limit = Some(bps);
                    self.engine.set_speed_limit(bps);
                    self.settings.save();
                }
                Command::none()
            }

            Message::ClearSpeedLimit => {
                self.speed_limit_input.clear();
                self.settings.speed_limit = None;
                self.engine.set_speed_limit(0);
                self.settings.save();
                Command::none()
            }

            Message::ToggleSchedule => {
                self.settings.schedule_enabled = !self.settings.schedule_enabled;
                self.settings.save();
                Command::none()
            }

            Message::SetScheduleFromH(val) => {
                self.sched_from_h = val.clone();
                if let Ok(h) = val.parse::<u8>() {
                    self.settings.schedule_from.0 = h.min(23);
                    self.settings.save();
                }
                Command::none()
            }
            Message::SetScheduleFromM(val) => {
                self.sched_from_m = val.clone();
                if let Ok(m) = val.parse::<u8>() {
                    self.settings.schedule_from.1 = m.min(59);
                    self.settings.save();
                }
                Command::none()
            }
            Message::SetScheduleToH(val) => {
                self.sched_to_h = val.clone();
                if let Ok(h) = val.parse::<u8>() {
                    self.settings.schedule_to.0 = h.min(23);
                    self.settings.save();
                }
                Command::none()
            }
            Message::SetScheduleToM(val) => {
                self.sched_to_m = val.clone();
                if let Ok(m) = val.parse::<u8>() {
                    self.settings.schedule_to.1 = m.min(59);
                    self.settings.save();
                }
                Command::none()
            }

            Message::WindowCloseRequested => {
                let (_total, active, _completed, paused, _failed) = self.counts;
                if active > 0 || paused > 0 {
                    self.save_downloads();
                    window::change_mode(window::Id::MAIN, window::Mode::Hidden)
                } else {
                    self.save_downloads();
                    window::close(window::Id::MAIN)
                }
            }

            Message::TrayShow => Command::batch([
                window::change_mode(window::Id::MAIN, window::Mode::Windowed),
                window::gain_focus(window::Id::MAIN),
            ]),

            Message::TrayQuit => {
                self.save_downloads();
                window::close(window::Id::MAIN)
            }

            Message::Noop => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        build_view(
            &self.downloads,
            self.filter,
            &self.url_input,
            self.selected,
            self.settings.theme_mode,
            self.total_speed,
            self.counts,
            &self.settings.download_dir,
            self.error_message.as_deref(),
            self.adding,
            self.view_mode,
            &self.settings,
            &self.speed_limit_input,
            &self.max_concurrent_input,
            &self.search_query,
            &self.sched_from_h,
            &self.sched_from_m,
            &self.sched_to_h,
            &self.sched_to_m,
            &self.history,
            self.network_online,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        let close_sub = event::listen_with(|e, _status| match e {
            Event::Window(_, window::Event::CloseRequested) => Some(Message::WindowCloseRequested),
            _ => None,
        });

        let has_active = self.counts.1 > 0;
        let has_failed = self.counts.4 > 0;
        let has_tray = self.tray.is_some();
        let has_scheduled_queued = self.settings.schedule_enabled
            && self.downloads.iter().any(|d| d.status == DownloadStatus::Queued);

        let tick_sub = if has_active {
            iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick)
        } else if has_tray || has_failed || has_scheduled_queued {
            iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick)
        } else {
            iced::time::every(Duration::from_secs(5)).map(|_| Message::Tick)
        };

        Subscription::batch([close_sub, tick_sub])
    }

    fn theme(&self) -> Theme {
        bolt_theme(self.settings.theme_mode)
    }
}

impl BoltApp {
    fn refresh_snapshots(&mut self) {
        let (snapshots, speed, counts) = self.engine.get_ui_state();
        self.downloads = snapshots;
        self.total_speed = speed;
        self.counts = counts;
    }

    fn save_downloads(&self) {
        let db = self.engine.persist();
        db.save();
    }

    fn check_newly_completed(&mut self, prev: &[DownloadItem]) {
        let mut changed = false;
        for dl in &self.downloads {
            if dl.status == DownloadStatus::Completed {
                let was_completed = prev
                    .iter()
                    .any(|p| p.id == dl.id && p.status == DownloadStatus::Completed);
                if !was_completed {
                    self.history.add(HistoryEntry {
                        id: dl.id,
                        url: dl.url.clone(),
                        filename: dl.filename.clone(),
                        save_path: dl.save_path.clone(),
                        total_size: dl.total_size,
                        category: dl.category,
                        completed_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                    });
                    changed = true;
                }
            }
        }
        if changed {
            self.history.save();
        }
    }

    fn record_completed_to_history(&mut self) {
        let mut changed = false;
        for dl in &self.downloads {
            if dl.status == DownloadStatus::Completed {
                self.history.add(HistoryEntry {
                    id: dl.id,
                    url: dl.url.clone(),
                    filename: dl.filename.clone(),
                    save_path: dl.save_path.clone(),
                    total_size: dl.total_size,
                    category: dl.category,
                    completed_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                });
                changed = true;
            }
        }
        if changed {
            self.history.save();
        }
    }

    fn update_tray_tooltip(&self) {
        if let Some(ref tray) = self.tray {
            let (total, active, completed, paused, _failed) = self.counts;
            let tip = if active > 0 {
                format!(
                    "Bolt - {} active at {} | {} total",
                    active,
                    format_speed(self.total_speed),
                    total
                )
            } else if paused > 0 {
                format!("Bolt - {} paused | {} total", paused, total)
            } else if completed > 0 {
                format!("Bolt - {} completed | {} total", completed, total)
            } else {
                "Bolt - Download Manager".to_string()
            };
            tray.set_tooltip(&tip);
        }
    }
}

fn open_path(path: &std::path::Path) -> std::io::Result<std::process::Child> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path).spawn()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "unsupported platform",
        ))
    }
}

