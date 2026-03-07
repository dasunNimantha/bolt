use crate::download::engine::DownloadEngine;
use crate::message::Message;
use crate::model::{DownloadFilter, DownloadItem, ViewMode};
use crate::settings::{AppSettings, DownloadDatabase};
use crate::theme::{bolt_theme, ThemeMode};
use crate::tray::BoltTray;
use crate::utils::format::format_speed;
use crate::view::build_view;
use iced::{event, window, Application, Command, Element, Event, Subscription, Theme};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct BoltApp {
    engine: Arc<DownloadEngine>,
    downloads: Vec<DownloadItem>,
    selected: Option<Uuid>,
    url_input: String,
    filter: DownloadFilter,
    settings: AppSettings,
    total_speed: f64,
    counts: (usize, usize, usize, usize, usize),
    error_message: Option<String>,
    adding: bool,
    view_mode: ViewMode,
    speed_limit_input: String,
    max_concurrent_input: String,
    persist_counter: u32,
    tray: Option<BoltTray>,
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

        let tray = BoltTray::new();

        let mut app = Self {
            engine,
            downloads: Vec::new(),
            selected: None,
            url_input: String::new(),
            filter: DownloadFilter::All,
            settings,
            total_speed: 0.0,
            counts: (0, 0, 0, 0, 0),
            error_message: None,
            adding: false,
            view_mode: ViewMode::Downloads,
            speed_limit_input,
            max_concurrent_input,
            persist_counter: 0,
            tray,
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
                self.engine.clear_completed();
                self.refresh_snapshots();
                self.save_downloads();
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

            Message::Tick => {
                self.engine.update_state();
                self.refresh_snapshots();
                self.update_tray_tooltip();

                self.persist_counter += 1;
                if self.persist_counter.is_multiple_of(8) {
                    self.save_downloads();

                    let due = self.engine.check_scheduled();
                    if !due.is_empty() {
                        let engine = self.engine.clone();
                        return Command::perform(
                            async move {
                                for id in due {
                                    let _ = engine.start_download(id).await;
                                }
                                Message::Tick
                            },
                            |msg| msg,
                        );
                    }

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

                if let Some(ref tray) = self.tray {
                    if let Some(action) = tray.poll() {
                        return match action {
                            crate::tray::TrayAction::Show => Command::batch([
                                window::minimize(window::Id::MAIN, false),
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

            Message::ScheduleDownload(id, datetime) => {
                self.engine.set_schedule(id, Some(datetime));
                self.refresh_snapshots();
                self.save_downloads();
                Command::none()
            }

            Message::ClearSchedule(id) => {
                self.engine.set_schedule(id, None);
                self.refresh_snapshots();
                self.save_downloads();
                Command::none()
            }

            Message::WindowCloseRequested => {
                let (_total, active, _completed, paused, _failed) = self.counts;
                if active > 0 || paused > 0 {
                    self.save_downloads();
                    window::minimize(window::Id::MAIN, true)
                } else {
                    self.save_downloads();
                    window::close(window::Id::MAIN)
                }
            }

            Message::TrayShow => Command::batch([
                window::minimize(window::Id::MAIN, false),
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
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        let close_sub = event::listen_with(|e, _status| match e {
            Event::Window(_, window::Event::CloseRequested) => Some(Message::WindowCloseRequested),
            _ => None,
        });

        let has_active = self.counts.1 > 0;
        let has_scheduled = self.downloads.iter().any(|d| {
            d.scheduled_at.is_some()
                && d.status == crate::model::DownloadStatus::Queued
        });
        let has_tray = self.tray.is_some();

        let tick_sub = if has_active {
            iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick)
        } else if has_scheduled || has_tray {
            iced::time::every(Duration::from_millis(2000)).map(|_| Message::Tick)
        } else {
            Subscription::none()
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
