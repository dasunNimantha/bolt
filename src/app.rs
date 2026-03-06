use crate::download::engine::DownloadEngine;
use crate::message::Message;
use crate::model::{DownloadFilter, DownloadItem};
use crate::settings::AppSettings;
use crate::theme::{bolt_theme, ThemeMode};
use crate::view::build_view;
use iced::{Application, Command, Element, Subscription, Theme};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct BoltApp {
    engine: Arc<DownloadEngine>,
    downloads: Vec<DownloadItem>,
    selected: Option<Uuid>,
    url_input: String,
    filter: DownloadFilter,
    theme_mode: ThemeMode,
    settings: AppSettings,
    total_speed: f64,
    counts: (usize, usize, usize, usize, usize),
    error_message: Option<String>,
}

impl Application for BoltApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let settings = AppSettings::load();
        let engine = Arc::new(DownloadEngine::new());

        let app = Self {
            engine,
            downloads: Vec::new(),
            selected: None,
            url_input: String::new(),
            filter: DownloadFilter::All,
            theme_mode: ThemeMode::Dark,
            settings,
            total_speed: 0.0,
            counts: (0, 0, 0, 0, 0),
            error_message: None,
        };

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
                self.error_message = None;
                self.refresh_snapshots();
                Command::none()
            }

            Message::DownloadError(err) => {
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
                        Message::Noop
                    },
                    |msg| msg,
                )
            }

            Message::PauseDownload(id) => {
                self.engine.pause(id);
                self.refresh_snapshots();
                Command::none()
            }

            Message::ResumeDownload(id) => {
                let engine = self.engine.clone();
                Command::perform(
                    async move {
                        let _ = engine.resume(id).await;
                        Message::Noop
                    },
                    |msg| msg,
                )
            }

            Message::CancelDownload(id) => {
                self.engine.cancel(id);
                self.refresh_snapshots();
                Command::none()
            }

            Message::RemoveDownload(id) => {
                self.engine.remove(id);
                if self.selected == Some(id) {
                    self.selected = None;
                }
                self.refresh_snapshots();
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
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                Command::none()
            }

            Message::Tick => {
                self.engine.update_state();
                self.refresh_snapshots();
                Command::none()
            }

            Message::OpenFile(id) => {
                if let Some(dl) = self.downloads.iter().find(|d| d.id == id) {
                    let path = dl.save_path.clone();
                    let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                }
                Command::none()
            }

            Message::OpenFolder(id) => {
                if let Some(dl) = self.downloads.iter().find(|d| d.id == id) {
                    if let Some(parent) = dl.save_path.parent() {
                        let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                    }
                }
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

            Message::Noop => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        build_view(
            &self.downloads,
            self.filter,
            &self.url_input,
            self.selected,
            self.theme_mode,
            self.total_speed,
            self.counts,
            &self.settings.download_dir,
            self.error_message.as_deref(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        bolt_theme(self.theme_mode)
    }
}

impl BoltApp {
    fn refresh_snapshots(&mut self) {
        self.downloads = self.engine.get_snapshots();
        self.total_speed = self.engine.total_speed();
        self.counts = self.engine.count_by_status();
    }
}
