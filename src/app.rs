use crate::download::engine::DownloadEngine;
use crate::message::Message;
use crate::model::{
    DownloadFilter, DownloadItem, DownloadStatus, HistoryEntry, PendingDownload, ViewMode,
};
use crate::settings::{AppSettings, DownloadDatabase, DownloadHistory, ProxyType};
use crate::theme::{bolt_theme, ThemeMode};
use crate::tray::BoltTray;
use crate::utils::format::format_speed;
use crate::view::build_view;
use chrono::Local;
use iced::{window, Element, Subscription, Task, Theme};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

const NETWORK_CHECK_INTERVAL: u32 = 12;

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
    proxy_host: String,
    proxy_port: String,
    proxy_user: String,
    proxy_pass: String,
    proxy_testing: bool,
    proxy_test_result: Option<Result<String, String>>,
    batch_status: Option<String>,
    persist_counter: u32,
    tray: Option<BoltTray>,
    network_online: bool,
    network_check_counter: u32,
    network_client: reqwest::Client,
    history: DownloadHistory,
    schedule_was_active: bool,
    ipc_pending: Arc<Mutex<Vec<PendingDownload>>>,
    popup_windows: HashMap<window::Id, PendingDownload>,
    main_window: Option<window::Id>,
    system_is_dark: bool,
}

impl BoltApp {
    pub fn boot() -> (Self, Task<Message>) {
        crate::nmh::auto_install();

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

        if let Some(proxy_url) = settings.proxy.to_url() {
            engine.set_proxy(Some(&proxy_url));
        }

        let proxy_host = settings.proxy.host.clone();
        let proxy_port = settings.proxy.port.clone();
        let proxy_user = settings.proxy.username.clone();
        let proxy_pass = settings.proxy.password.clone();

        let tray = BoltTray::new();
        let history = DownloadHistory::load();
        let schedule_was_active = settings.schedule_enabled && settings.is_within_schedule();

        let ipc_pending: Arc<Mutex<Vec<PendingDownload>>> = Arc::new(Mutex::new(Vec::new()));

        let ipc_task = {
            let pending = ipc_pending.clone();
            Task::perform(
                async move {
                    crate::ipc::start_ipc_server(pending).await;
                    Message::Noop
                },
                |msg| msg,
            )
        };

        let (main_id, open_main) = window::open(window::Settings {
            size: iced::Size::new(1000.0, 650.0),
            min_size: Some(iced::Size::new(750.0, 450.0)),
            exit_on_close_request: false,
            #[cfg(target_os = "linux")]
            platform_specific: window::settings::PlatformSpecific {
                application_id: "com.bolt.downloadmanager".to_string(),
                ..Default::default()
            },
            ..Default::default()
        });

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
            proxy_host,
            proxy_port,
            proxy_user,
            proxy_pass,
            proxy_testing: false,
            proxy_test_result: None,
            batch_status: None,
            persist_counter: 0,
            tray,
            network_online: true,
            network_check_counter: 0,
            network_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .pool_max_idle_per_host(0)
                .build()
                .unwrap_or_default(),
            history,
            schedule_was_active,
            ipc_pending,
            popup_windows: HashMap::new(),
            main_window: Some(main_id),
            system_is_dark: true,
        };
        app.refresh_snapshots();

        let system_theme_task = iced::system::theme()
            .map(|mode| Message::SystemThemeChanged(mode == iced::theme::Mode::Dark));

        (
            app,
            Task::batch([
                ipc_task,
                open_main.map(|_| Message::Noop),
                system_theme_task,
            ]),
        )
    }

    pub fn title(&self, id: window::Id) -> String {
        if self.popup_windows.contains_key(&id) {
            return "Bolt - New Download".to_string();
        }
        let (total, active, ..) = self.counts;
        if active > 0 {
            format!("Bolt - {} active / {} total", active, total)
        } else if total > 0 {
            format!("Bolt - {} downloads", total)
        } else {
            "Bolt - Download Manager".to_string()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UrlInputChanged(url) => {
                self.url_input = url;
                self.error_message = None;
                Task::none()
            }

            Message::AddDownload => {
                let raw = self.url_input.trim().to_string();
                if raw.is_empty() {
                    return Task::none();
                }

                let urls: Vec<String> = raw
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .map(|u| {
                        if !u.starts_with("http://") && !u.starts_with("https://") {
                            format!("https://{}", u)
                        } else {
                            u
                        }
                    })
                    .filter(|u| url::Url::parse(u).is_ok())
                    .collect();

                if urls.is_empty() {
                    self.error_message = Some("No valid URLs found".to_string());
                    return Task::none();
                }

                self.url_input.clear();
                self.adding = true;

                let engine = self.engine.clone();
                let save_dir = self.settings.download_dir.clone();

                if urls.len() == 1 {
                    let url = urls.into_iter().next().unwrap();
                    Task::perform(
                        async move {
                            match engine.add_download(url, save_dir).await {
                                Ok(item) => Message::DownloadAdded(Box::new(item)),
                                Err(e) => Message::DownloadError(e.to_string()),
                            }
                        },
                        |msg| msg,
                    )
                } else {
                    Task::perform(
                        async move {
                            let total = urls.len();
                            let mut ok = 0usize;
                            for url in urls {
                                if engine.add_download(url, save_dir.clone()).await.is_ok() {
                                    ok += 1;
                                }
                            }
                            Message::BatchAddResult(ok, total)
                        },
                        |msg| msg,
                    )
                }
            }

            Message::DownloadAdded(_item) => {
                self.adding = false;
                self.error_message = None;
                self.refresh_snapshots();
                self.save_downloads();
                Task::none()
            }

            Message::DownloadError(err) => {
                self.adding = false;
                eprintln!("Download error: {}", err);
                self.error_message = Some(err);
                Task::none()
            }

            Message::StartDownload(id) => {
                let engine = self.engine.clone();
                Task::perform(
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
                Task::none()
            }

            Message::ResumeDownload(id) => {
                let engine = self.engine.clone();
                Task::perform(
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
                Task::none()
            }

            Message::RemoveDownload(id) => {
                self.engine.remove(id);
                if self.selected == Some(id) {
                    self.selected = None;
                }
                self.refresh_snapshots();
                self.save_downloads();
                Task::none()
            }

            Message::RetryDownload(id) => {
                let engine = self.engine.clone();
                Task::perform(
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
                Task::none()
            }

            Message::SelectDownload(id) => {
                self.selected = id;
                Task::none()
            }

            Message::FilterChanged(filter) => {
                self.filter = filter;
                Task::none()
            }

            Message::ToggleTheme => {
                self.settings.theme_mode = match self.settings.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::System,
                    ThemeMode::System => ThemeMode::Dark,
                };
                self.settings.save();
                Task::none()
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
                Task::none()
            }

            Message::WindowOpened(id) => {
                if self.main_window.is_none() {
                    self.main_window = Some(id);
                }
                Task::none()
            }

            Message::Tick => {
                self.engine.update_state();

                {
                    let new_pending: Vec<PendingDownload> = {
                        let mut pending = self.ipc_pending.lock().unwrap();
                        pending.drain(..).collect()
                    };

                    if !new_pending.is_empty() {
                        let mut tasks = Vec::new();
                        for p in new_pending {
                            let (win_id, open_task) = window::open(window::Settings {
                                size: iced::Size::new(620.0, 210.0),
                                min_size: Some(iced::Size::new(620.0, 210.0)),
                                max_size: Some(iced::Size::new(620.0, 210.0)),
                                resizable: false,
                                level: window::Level::AlwaysOnTop,
                                exit_on_close_request: false,
                                ..Default::default()
                            });
                            let engine = self.engine.clone();
                            let url = p.url.clone();
                            let hdrs = p.headers.clone();
                            let resolve_task = Task::perform(
                                async move {
                                    match engine.resolve_file_info(&url, &hdrs).await {
                                        Ok(info) => {
                                            Message::IpcResolved(win_id, Box::new(info))
                                        }
                                        Err(_) => Message::Noop,
                                    }
                                },
                                |msg| msg,
                            );
                            self.popup_windows.insert(win_id, p);
                            tasks.push(open_task.map(|_| Message::Noop));
                            tasks.push(resolve_task);
                        }
                        return Task::batch(tasks);
                    }
                }

                let prev_completed: std::collections::HashSet<Uuid> = self
                    .downloads
                    .iter()
                    .filter(|d| d.status == DownloadStatus::Completed)
                    .map(|d| d.id)
                    .collect();
                self.refresh_snapshots();
                self.check_newly_completed(&prev_completed);

                self.update_tray_tooltip();

                self.persist_counter += 1;
                if self.persist_counter.is_multiple_of(8) {
                    self.save_downloads();

                    let in_window =
                        self.settings.schedule_enabled && self.settings.is_within_schedule();
                    if in_window && !self.schedule_was_active {
                        let queued = self.engine.get_queued_ids();
                        if !queued.is_empty() {
                            self.schedule_was_active = true;
                            let engine = self.engine.clone();
                            return Task::perform(
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

                    let auto_start = self.engine.auto_start_queued();
                    if !auto_start.is_empty() {
                        let engine = self.engine.clone();
                        return Task::perform(
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

                self.network_check_counter += 1;
                if self.network_check_counter >= NETWORK_CHECK_INTERVAL {
                    self.network_check_counter = 0;
                    let client = self.network_client.clone();
                    return Task::perform(
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
                            crate::tray::TrayAction::Show => Task::batch([
                                window::set_mode(self.main_window.unwrap(), window::Mode::Windowed),
                                window::gain_focus(self.main_window.unwrap()),
                            ]),
                            crate::tray::TrayAction::Quit => {
                                self.save_downloads();
                                Task::batch([
                                    window::close(self.main_window.unwrap()),
                                    iced::exit(),
                                ])
                            }
                        };
                    }
                }

                Task::none()
            }

            Message::NetworkStatus(online) => {
                let was_offline = !self.network_online;
                self.network_online = online;

                if online && was_offline {
                    let failed_ids = self.engine.get_failed_ids();
                    if !failed_ids.is_empty() {
                        let engine = self.engine.clone();
                        return Task::perform(
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

                Task::none()
            }

            Message::OpenFile(id) => {
                if let Some(dl) = self.downloads.iter().find(|d| d.id == id) {
                    let _ = open_path(&dl.save_path);
                }
                Task::none()
            }

            Message::OpenFolder(id) => {
                if let Some(dl) = self.downloads.iter().find(|d| d.id == id) {
                    if let Some(parent) = dl.save_path.parent() {
                        let _ = open_path(parent);
                    }
                }
                Task::none()
            }

            Message::ShowSettings => {
                self.view_mode = ViewMode::Settings;
                Task::none()
            }

            Message::ShowDownloads => {
                self.view_mode = ViewMode::Downloads;
                Task::none()
            }

            Message::ChooseDownloadDir => Task::perform(
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
                Task::none()
            }

            Message::SetMaxConcurrent(val) => {
                self.max_concurrent_input = val;
                if let Ok(n) = self.max_concurrent_input.parse::<usize>() {
                    let n = n.clamp(1, 10);
                    self.settings.max_concurrent = n;
                    self.engine.set_max_concurrent(n as u64);
                    self.settings.save();
                }
                Task::none()
            }

            Message::SetSpeedLimit(val) => {
                self.speed_limit_input = val;
                if self.speed_limit_input.is_empty() {
                    self.settings.speed_limit = None;
                    self.engine.set_speed_limit(0);
                    self.settings.save();
                } else if let Ok(kb) = self.speed_limit_input.parse::<u64>() {
                    let bps = kb * 1024;
                    self.settings.speed_limit = Some(bps);
                    self.engine.set_speed_limit(bps);
                    self.settings.save();
                }
                Task::none()
            }

            Message::ClearSpeedLimit => {
                self.speed_limit_input.clear();
                self.settings.speed_limit = None;
                self.engine.set_speed_limit(0);
                self.settings.save();
                Task::none()
            }

            Message::ToggleSchedule => {
                self.settings.schedule_enabled = !self.settings.schedule_enabled;
                self.settings.save();
                Task::none()
            }

            Message::SetScheduleFromH(val) => {
                self.sched_from_h = val;
                if let Ok(h) = self.sched_from_h.parse::<u8>() {
                    self.settings.schedule_from.0 = h.min(23);
                    self.settings.save();
                }
                Task::none()
            }
            Message::SetScheduleFromM(val) => {
                self.sched_from_m = val;
                if let Ok(m) = self.sched_from_m.parse::<u8>() {
                    self.settings.schedule_from.1 = m.min(59);
                    self.settings.save();
                }
                Task::none()
            }
            Message::SetScheduleToH(val) => {
                self.sched_to_h = val;
                if let Ok(h) = self.sched_to_h.parse::<u8>() {
                    self.settings.schedule_to.0 = h.min(23);
                    self.settings.save();
                }
                Task::none()
            }
            Message::SetScheduleToM(val) => {
                self.sched_to_m = val;
                if let Ok(m) = self.sched_to_m.parse::<u8>() {
                    self.settings.schedule_to.1 = m.min(59);
                    self.settings.save();
                }
                Task::none()
            }

            Message::WindowCloseRequested(id) => {
                if self.popup_windows.remove(&id).is_some() {
                    return window::close(id);
                }
                self.save_downloads();
                if self.tray.is_some() {
                    window::set_mode(id, window::Mode::Hidden)
                } else {
                    Task::batch([window::close(id), iced::exit()])
                }
            }

            Message::TrayShow => Task::batch([
                window::set_mode(self.main_window.unwrap(), window::Mode::Windowed),
                window::gain_focus(self.main_window.unwrap()),
            ]),

            Message::TrayQuit => {
                self.save_downloads();
                Task::batch([window::close(self.main_window.unwrap()), iced::exit()])
            }

            Message::ImportFile => Task::perform(
                async {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Import URLs from text file")
                        .add_filter("Text files", &["txt"])
                        .pick_file()
                        .await;
                    Message::ImportFileChosen(handle.map(|h| h.path().to_path_buf()))
                },
                |msg| msg,
            ),

            Message::ImportFileChosen(path) => {
                if let Some(file_path) = path {
                    if let Ok(contents) = std::fs::read_to_string(&file_path) {
                        let urls: Vec<String> = contents
                            .lines()
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty() && !l.starts_with('#'))
                            .map(|u| {
                                if !u.starts_with("http://") && !u.starts_with("https://") {
                                    format!("https://{}", u)
                                } else {
                                    u
                                }
                            })
                            .filter(|u| url::Url::parse(u).is_ok())
                            .collect();

                        if urls.is_empty() {
                            self.error_message = Some("No valid URLs found in file".to_string());
                            return Task::none();
                        }

                        self.adding = true;
                        let engine = self.engine.clone();
                        let save_dir = self.settings.download_dir.clone();

                        return Task::perform(
                            async move {
                                let total = urls.len();
                                let mut ok = 0usize;
                                for url in urls {
                                    if engine.add_download(url, save_dir.clone()).await.is_ok() {
                                        ok += 1;
                                    }
                                }
                                Message::BatchAddResult(ok, total)
                            },
                            |msg| msg,
                        );
                    }
                    self.error_message = Some("Could not read file".to_string());
                }
                Task::none()
            }

            Message::BatchAddResult(ok, total) => {
                self.adding = false;
                let failed = total - ok;
                if failed > 0 {
                    self.error_message = Some(format!(
                        "Added {} of {} URLs ({} failed)",
                        ok, total, failed
                    ));
                } else {
                    self.error_message = None;
                    self.batch_status = Some(format!("Added {} downloads", ok));
                }
                self.refresh_snapshots();
                self.save_downloads();
                Task::none()
            }

            Message::SetProxyType(pt) => {
                self.settings.proxy.proxy_type = pt;
                self.proxy_test_result = None;
                if pt == ProxyType::None {
                    self.engine.set_proxy(None);
                } else {
                    self.apply_proxy();
                }
                self.settings.save();
                Task::none()
            }

            Message::SetProxyHost(val) => {
                self.proxy_host = val;
                self.settings.proxy.host = self.proxy_host.clone();
                self.proxy_test_result = None;
                self.apply_proxy();
                self.settings.save();
                Task::none()
            }

            Message::SetProxyPort(val) => {
                self.proxy_port = val;
                self.settings.proxy.port = self.proxy_port.clone();
                self.proxy_test_result = None;
                self.apply_proxy();
                self.settings.save();
                Task::none()
            }

            Message::SetProxyUser(val) => {
                self.proxy_user = val;
                self.settings.proxy.username = self.proxy_user.clone();
                self.proxy_test_result = None;
                self.apply_proxy();
                self.settings.save();
                Task::none()
            }

            Message::SetProxyPass(val) => {
                self.proxy_pass = val;
                self.settings.proxy.password = self.proxy_pass.clone();
                self.proxy_test_result = None;
                self.apply_proxy();
                self.settings.save();
                Task::none()
            }

            Message::TestProxy => {
                if !self.settings.proxy.is_active() {
                    self.proxy_test_result = Some(Err("Configure proxy host first".to_string()));
                    return Task::none();
                }
                self.proxy_testing = true;
                self.proxy_test_result = None;
                let proxy_url = self.settings.proxy.to_url().unwrap();
                Task::perform(
                    async move {
                        let proxy = match reqwest::Proxy::all(&proxy_url) {
                            Ok(p) => p,
                            Err(e) => {
                                return Message::ProxyTestResult(Err(format!(
                                    "Invalid proxy URL: {}",
                                    e
                                )));
                            }
                        };
                        let client = match reqwest::Client::builder()
                            .proxy(proxy)
                            .timeout(Duration::from_secs(10))
                            .build()
                        {
                            Ok(c) => c,
                            Err(e) => {
                                return Message::ProxyTestResult(Err(format!(
                                    "Client error: {}",
                                    e
                                )));
                            }
                        };
                        match client
                            .get("http://ip-api.com/json/?fields=country")
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(json) => {
                                        let country = json["country"]
                                            .as_str()
                                            .unwrap_or("Unknown")
                                            .to_string();
                                        Message::ProxyTestResult(Ok(country))
                                    }
                                    Err(e) => Message::ProxyTestResult(Err(format!(
                                        "Failed to parse response: {}",
                                        e
                                    ))),
                                }
                            }
                            Ok(resp) => {
                                Message::ProxyTestResult(Err(format!("HTTP {}", resp.status())))
                            }
                            Err(e) => {
                                let msg = e.to_string();
                                let short = if let Some(pos) = msg.find("error trying to connect") {
                                    let rest = &msg[pos..];
                                    if let Some(colon) = rest.rfind(": ") {
                                        format!("Connection failed: {}", &rest[colon + 2..])
                                    } else {
                                        format!("Connection failed: {}", rest)
                                    }
                                } else {
                                    format!("Connection failed: {}", msg)
                                };
                                Message::ProxyTestResult(Err(short))
                            }
                        }
                    },
                    |msg| msg,
                )
            }

            Message::ProxyTestResult(result) => {
                self.proxy_testing = false;
                self.proxy_test_result = Some(result);
                Task::none()
            }

            Message::IpcResolved(popup_id, info) => {
                if let Some(pending) = self.popup_windows.get_mut(&popup_id) {
                    pending.resolved = Some(*info);
                }
                Task::none()
            }

            Message::IpcAcceptStart(popup_id) => {
                if let Some(pending) = self.popup_windows.remove(&popup_id) {
                    self.adding = true;
                    let engine = self.engine.clone();
                    let save_dir = self.settings.download_dir.clone();
                    let headers = if pending.headers.is_empty() {
                        None
                    } else {
                        Some(pending.headers)
                    };
                    let add_task = if let Some(info) = pending.resolved {
                        Task::perform(
                            async move {
                                match engine
                                    .add_download_resolved(pending.url, save_dir, headers, info)
                                    .await
                                {
                                    Ok(item) => {
                                        let id = item.id;
                                        let _ = engine.start_download(id).await;
                                        Message::DownloadAdded(Box::new(item))
                                    }
                                    Err(e) => Message::DownloadError(e.to_string()),
                                }
                            },
                            |msg| msg,
                        )
                    } else {
                        Task::perform(
                            async move {
                                match engine
                                    .add_download_with_headers(pending.url, save_dir, headers)
                                    .await
                                {
                                    Ok(item) => {
                                        let id = item.id;
                                        let _ = engine.start_download(id).await;
                                        Message::DownloadAdded(Box::new(item))
                                    }
                                    Err(e) => Message::DownloadError(e.to_string()),
                                }
                            },
                            |msg| msg,
                        )
                    };
                    return Task::batch([
                        window::close(popup_id),
                        window::set_mode(self.main_window.unwrap(), window::Mode::Windowed),
                        window::gain_focus(self.main_window.unwrap()),
                        add_task,
                    ]);
                }
                Task::none()
            }

            Message::IpcAcceptQueue(popup_id) => {
                if let Some(pending) = self.popup_windows.remove(&popup_id) {
                    self.adding = true;
                    let engine = self.engine.clone();
                    let save_dir = self.settings.download_dir.clone();
                    let headers = if pending.headers.is_empty() {
                        None
                    } else {
                        Some(pending.headers)
                    };
                    let add_task = if let Some(info) = pending.resolved {
                        Task::perform(
                            async move {
                                match engine
                                    .add_download_resolved(pending.url, save_dir, headers, info)
                                    .await
                                {
                                    Ok(item) => Message::DownloadAdded(Box::new(item)),
                                    Err(e) => Message::DownloadError(e.to_string()),
                                }
                            },
                            |msg| msg,
                        )
                    } else {
                        Task::perform(
                            async move {
                                match engine
                                    .add_download_with_headers(pending.url, save_dir, headers)
                                    .await
                                {
                                    Ok(item) => Message::DownloadAdded(Box::new(item)),
                                    Err(e) => Message::DownloadError(e.to_string()),
                                }
                            },
                            |msg| msg,
                        )
                    };
                    return Task::batch([
                        window::close(popup_id),
                        window::set_mode(self.main_window.unwrap(), window::Mode::Windowed),
                        window::gain_focus(self.main_window.unwrap()),
                        add_task,
                    ]);
                }
                Task::none()
            }

            Message::IpcDismiss(popup_id) => {
                self.popup_windows.remove(&popup_id);
                window::close(popup_id)
            }

            Message::SystemThemeChanged(is_dark) => {
                self.system_is_dark = is_dark;
                Task::none()
            }

            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        let effective_theme = self.settings.theme_mode.effective(self.system_is_dark);

        if let Some(pending) = self.popup_windows.get(&id) {
            return crate::view::build_popup_window_view(
                pending,
                effective_theme,
                &self.settings.download_dir,
                id,
            );
        }

        build_view(
            &self.downloads,
            self.filter,
            &self.url_input,
            self.selected,
            effective_theme,
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
            &self.proxy_host,
            &self.proxy_port,
            &self.proxy_user,
            &self.proxy_pass,
            self.proxy_testing,
            self.proxy_test_result.as_ref(),
            &self.history,
            self.network_online,
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let open_sub = window::open_events().map(Message::WindowOpened);
        let close_sub = window::close_requests().map(Message::WindowCloseRequested);

        let has_active = self.counts.1 > 0;
        let has_failed = self.counts.4 > 0;
        let has_tray = self.tray.is_some();
        let has_popup_pending = !self.popup_windows.is_empty();
        let has_scheduled_queued = self.settings.schedule_enabled
            && self
                .downloads
                .iter()
                .any(|d| d.status == DownloadStatus::Queued);

        let tick_sub = if has_active || has_popup_pending {
            iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick)
        } else if has_tray || has_failed || has_scheduled_queued {
            iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick)
        } else {
            iced::time::every(Duration::from_secs(5)).map(|_| Message::Tick)
        };

        let system_theme_sub = iced::system::theme_changes()
            .map(|mode| Message::SystemThemeChanged(mode == iced::theme::Mode::Dark));

        Subscription::batch([open_sub, close_sub, tick_sub, system_theme_sub])
    }

    pub fn theme(&self, _id: window::Id) -> Theme {
        bolt_theme(self.settings.theme_mode.effective(self.system_is_dark))
    }

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

    fn check_newly_completed(&mut self, prev_completed: &std::collections::HashSet<Uuid>) {
        let mut changed = false;
        for dl in &self.downloads {
            if dl.status == DownloadStatus::Completed && !prev_completed.contains(&dl.id) {
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

    fn apply_proxy(&self) {
        if let Some(url) = self.settings.proxy.to_url() {
            self.engine.set_proxy(Some(&url));
        } else {
            self.engine.set_proxy(None);
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
