use crate::message::Message;
use crate::model::{
    DownloadFilter, DownloadItem, DownloadStatus, FileCategory, PendingDownload, ViewMode,
};
use crate::settings::{AppSettings, DownloadHistory, ProxyType};
use crate::theme::{
    card_style, danger_button, download_card_style, filter_button, get_colors, icon_button,
    panel_style, primary_button, progress_complete_style, progress_error_style,
    progress_paused_style, progress_style, scrollable_style, secondary_button, status_badge_style,
    text_input_style, transparent_button, ColorScheme, ThemeMode,
};
use crate::utils::format::{
    format_bytes, format_eta, format_speed, truncate_filename, truncate_url,
};
use iced::widget::{
    button, column, container, progress_bar, row, scrollable, text, text_input, Column, Row, Space,
};
use iced::{window, Alignment, Background, Border, Color, Element, Font, Length, Theme};
use iced_fonts::bootstrap as bs;

const JETBRAINS_MONO: Font = Font::with_name("JetBrains Mono");

fn icon(f: fn() -> iced::widget::Text<'static>) -> iced::widget::Text<'static> {
    f().size(17.0)
}

fn icon_sized(f: fn() -> iced::widget::Text<'static>, size: f32) -> iced::widget::Text<'static> {
    f().size(size)
}

#[allow(clippy::too_many_arguments)]
pub fn build_view<'a>(
    downloads: &'a [DownloadItem],
    filter: DownloadFilter,
    url_input: &'a str,
    selected: Option<uuid::Uuid>,
    theme_mode: ThemeMode,
    total_speed: f64,
    counts: (usize, usize, usize, usize, usize),
    download_dir: &'a std::path::Path,
    error_message: Option<&'a str>,
    adding: bool,
    view_mode: ViewMode,
    settings: &'a AppSettings,
    speed_limit_input: &'a str,
    max_concurrent_input: &'a str,
    search_query: &'a str,
    sched_from_h: &'a str,
    sched_from_m: &'a str,
    sched_to_h: &'a str,
    sched_to_m: &'a str,
    proxy_host: &'a str,
    proxy_port: &'a str,
    proxy_user: &'a str,
    proxy_pass: &'a str,
    proxy_testing: bool,
    proxy_test_result: Option<&'a Result<String, String>>,
    history: &'a DownloadHistory,
    network_online: bool,
) -> Element<'a, Message> {
    let colors = get_colors(theme_mode);
    let is_dark = theme_mode == ThemeMode::Dark;

    let header = build_header(colors, is_dark, view_mode);

    let body: Element<'a, Message> = match view_mode {
        ViewMode::Downloads => build_downloads_view(
            downloads,
            filter,
            url_input,
            selected,
            colors,
            is_dark,
            total_speed,
            counts,
            download_dir,
            error_message,
            adding,
            search_query,
            history,
            network_online,
        ),
        ViewMode::Settings => build_settings_view(
            colors,
            is_dark,
            settings,
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
            proxy_testing,
            proxy_test_result,
        ),
    };

    let content = Column::new()
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .push(header)
        .push(Space::new().height(10))
        .push(body);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([12, 16])
        .style(move |_theme: &Theme| container::Style {
            text_color: Some(colors.text_primary),
            background: Some(Background::Color(colors.bg_primary)),
            border: Border::default(),
            shadow: Default::default(),
            ..Default::default()
        })
        .into()
}

fn build_header(
    colors: ColorScheme,
    is_dark: bool,
    view_mode: ViewMode,
) -> Element<'static, Message> {
    let nav_button = match view_mode {
        ViewMode::Downloads => button(icon(bs::gear_fill))
            .on_press(Message::ShowSettings)
            .padding([6, 8])
            .style(icon_button(colors)),
        ViewMode::Settings => button(
            row![
                icon(bs::arrow_left),
                Space::new().width(6),
                text("Back").size(14),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::ShowDownloads)
        .padding([6, 12])
        .style(secondary_button(colors)),
    };

    container(
        row![
            icon_sized(bs::lightning_charge_fill, 26.0).color(colors.accent_primary),
            Space::new().width(10),
            text("Bolt").size(24).color(colors.text_primary),
            Space::new().width(Length::Fill),
            nav_button,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([10, 16])
    .style(card_style(colors, is_dark))
    .into()
}

pub fn build_popup_window_view<'a>(
    pending: &'a PendingDownload,
    theme_mode: ThemeMode,
    download_dir: &'a std::path::Path,
    popup_id: window::Id,
) -> Element<'a, Message> {
    let colors = get_colors(theme_mode);

    let filename = pending.display_filename();
    let category = FileCategory::from_filename(&filename);

    let category_icon: fn() -> iced::widget::Text<'static> = match category {
        FileCategory::Video => bs::camera_video,
        FileCategory::Audio => bs::music_note_beamed,
        FileCategory::Document => bs::file_text,
        FileCategory::Archive => bs::file_zip,
        FileCategory::Image => bs::image,
        FileCategory::Application => bs::terminal,
        FileCategory::Other => bs::file_earmark,
    };

    let dir_display = download_dir.to_str().unwrap_or("Downloads").to_string();
    let url_display = truncate_url(&pending.url, 60);

    let title_row = row![
        icon_sized(bs::download, 20.0).color(colors.accent_primary),
        Space::new().width(8),
        text("New Download").size(17).color(colors.text_primary),
    ]
    .align_y(Alignment::Center);

    let divider = container(Space::new().height(0))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_theme: &Theme| container::Style {
            text_color: None,
            background: Some(Background::Color(colors.border_light)),
            border: Border::default(),
            shadow: Default::default(),
            ..Default::default()
        });

    let size_label: Element<'a, Message> = if let Some(ref info) = pending.resolved {
        if let Some(size) = info.total_size {
            text(format_bytes(size)).size(12).color(colors.text_secondary).into()
        } else {
            text("Unknown size").size(12).color(colors.text_disabled).into()
        }
    } else {
        text("Resolving...").size(12).color(colors.text_disabled).into()
    };

    let file_row = row![
        icon(category_icon).color(colors.accent_primary),
        Space::new().width(8),
        text(truncate_filename(&filename, 60))
            .size(14)
            .color(colors.text_primary),
        Space::new().width(Length::Fill),
        size_label,
        Space::new().width(8),
        container(text(category.label()).size(11))
            .padding([2, 8])
            .style(status_badge_style(colors.accent_primary)),
    ]
    .align_y(Alignment::Center);

    let url_row = row![
        icon(bs::link).color(colors.text_disabled),
        Space::new().width(8),
        text(url_display).size(12).color(colors.text_secondary),
    ]
    .align_y(Alignment::Center);

    let dir_row = row![
        icon(bs::folder).color(colors.text_disabled),
        Space::new().width(8),
        text("Save to:").size(12).color(colors.text_secondary),
        Space::new().width(4),
        text(dir_display).size(12).color(colors.text_primary),
    ]
    .align_y(Alignment::Center);

    let start_btn = button(
        row![
            icon(bs::play_fill).color(Color::from_rgb(0.1, 0.1, 0.1)),
            Space::new().width(5),
            text("Start Download").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::IpcAcceptStart(popup_id))
    .padding([7, 16])
    .style(primary_button(colors));

    let queue_btn = button(
        row![
            icon(bs::plus).size(14),
            Space::new().width(5),
            text("Add to Queue").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::IpcAcceptQueue(popup_id))
    .padding([7, 16])
    .style(secondary_button(colors));

    let cancel_btn = button(
        row![
            icon(bs::x_lg).size(12),
            Space::new().width(5),
            text("Cancel").size(13),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::IpcDismiss(popup_id))
    .padding([7, 16])
    .style(danger_button(colors));

    let button_row = row![
        start_btn,
        Space::new().width(8),
        queue_btn,
        Space::new().width(Length::Fill),
        cancel_btn,
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let content = column![
        title_row,
        Space::new().height(8),
        divider,
        Space::new().height(8),
        file_row,
        Space::new().height(4),
        url_row,
        Space::new().height(4),
        dir_row,
        Space::new().height(12),
        button_row,
    ]
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([16, 22])
        .style(move |_theme: &Theme| container::Style {
            text_color: Some(colors.text_primary),
            background: Some(Background::Color(colors.bg_primary)),
            border: Border::default(),
            shadow: Default::default(),
            ..Default::default()
        })
        .into()
}

#[allow(clippy::too_many_arguments)]
fn build_downloads_view<'a>(
    downloads: &'a [DownloadItem],
    filter: DownloadFilter,
    url_input: &'a str,
    selected: Option<uuid::Uuid>,
    colors: ColorScheme,
    is_dark: bool,
    total_speed: f64,
    counts: (usize, usize, usize, usize, usize),
    download_dir: &'a std::path::Path,
    error_message: Option<&'a str>,
    adding: bool,
    search_query: &'a str,
    _history: &'a DownloadHistory,
    network_online: bool,
) -> Element<'a, Message> {
    let url_bar = build_url_bar(url_input, colors, adding);
    let filter_bar = build_filter_bar(filter, colors, counts);

    let query_lower = search_query.to_lowercase();
    let filtered: Vec<&DownloadItem> = downloads
        .iter()
        .filter(|d| filter.matches(d.status))
        .filter(|d| {
            if search_query.is_empty() {
                return true;
            }
            d.filename.to_lowercase().contains(&query_lower)
                || d.url.to_lowercase().contains(&query_lower)
        })
        .collect();

    let download_list = build_download_list(&filtered, selected, colors, is_dark);
    let status_bar = build_status_bar(total_speed, counts, colors, download_dir, network_online);

    let mut content = Column::new()
        .spacing(6)
        .width(Length::Fill)
        .height(Length::Fill)
        .push(url_bar);

    if adding {
        content = content.push(
            container(
                row![
                    icon(bs::hourglass_split).color(colors.accent_primary),
                    Space::new().width(10),
                    text("Fetching file info...")
                        .size(14)
                        .color(colors.accent_primary),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([10, 20])
            .style(move |_theme: &Theme| container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgba(0.95, 0.75, 0.25, 0.08))),
                border: Border {
                    color: colors.accent_primary,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                ..Default::default()
            }),
        );
    }

    if let Some(err) = error_message {
        content = content.push(
            container(
                row![
                    icon(bs::exclamation_triangle_fill).color(colors.error),
                    Space::new().width(8),
                    text(err).size(14).color(colors.error),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 20])
            .style(move |_theme: &Theme| container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgba(0.95, 0.35, 0.35, 0.1))),
                border: Border {
                    color: colors.error,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                ..Default::default()
            }),
        );
    }

    if !network_online {
        content = content.push(
            container(
                row![
                    icon(bs::wifi_off).color(colors.warning),
                    Space::new().width(8),
                    text(
                        "Network offline — downloads will auto-resume when connection is restored"
                    )
                    .size(13)
                    .color(colors.warning),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 20])
            .style(move |_theme: &Theme| container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgba(0.95, 0.75, 0.25, 0.06))),
                border: Border {
                    color: colors.warning,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                ..Default::default()
            }),
        );
    }

    let search_bar = build_search_bar(search_query, colors);

    content = content
        .push(
            row![filter_bar, search_bar]
                .align_y(Alignment::Center)
                .spacing(8)
                .width(Length::Fill),
        )
        .push(download_list)
        .push(status_bar);

    content.into()
}

#[allow(clippy::too_many_arguments)]
fn build_settings_view<'a>(
    colors: ColorScheme,
    is_dark: bool,
    settings: &'a AppSettings,
    speed_limit_input: &'a str,
    max_concurrent_input: &'a str,
    sched_from_h: &'a str,
    sched_from_m: &'a str,
    sched_to_h: &'a str,
    sched_to_m: &'a str,
    _proxy_host: &'a str,
    _proxy_port: &'a str,
    _proxy_user: &'a str,
    _proxy_pass: &'a str,
    _proxy_testing: bool,
    _proxy_test_result: Option<&'a Result<String, String>>,
) -> Element<'a, Message> {
    let dir_display = settings.download_dir.to_str().unwrap_or("Unknown");

    let speed_status = if speed_limit_input.is_empty() {
        "Unlimited".to_string()
    } else if let Ok(kb) = speed_limit_input.parse::<u64>() {
        format!("{}/s", format_bytes(kb * 1024))
    } else {
        "Invalid value".to_string()
    };

    let sections = Column::new()
        .spacing(20)
        .width(Length::Fill)
        // General
        .push(settings_group(
            colors,
            is_dark,
            "General",
            vec![
                settings_row(
                    colors,
                    "Theme",
                    button(
                        row![
                            icon(match settings.theme_mode {
                                ThemeMode::Dark => bs::moon_stars_fill as fn() -> _,
                                ThemeMode::Light => bs::sun_fill as fn() -> _,
                                ThemeMode::System => bs::display as fn() -> _,
                            }),
                            Space::new().width(6),
                            text(settings.theme_mode.label()).size(14),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::ToggleTheme)
                    .padding([6, 14])
                    .style(secondary_button(colors))
                    .into(),
                    None,
                    true,
                ),
                settings_row(
                    colors,
                    "Download directory",
                    button(
                        row![
                            icon(bs::folder_symlink).size(14),
                            Space::new().width(6),
                            text("Change").size(14),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::ChooseDownloadDir)
                    .padding([6, 14])
                    .style(secondary_button(colors))
                    .into(),
                    Some(dir_display.to_string()),
                    false,
                ),
            ],
        ))
        // Downloads
        .push(settings_group(
            colors,
            is_dark,
            "Downloads",
            vec![
                settings_row(
                    colors,
                    "Concurrent downloads",
                    row![
                        text_input("3", max_concurrent_input)
                            .on_input(Message::SetMaxConcurrent)
                            .padding([6, 10])
                            .size(14)
                            .width(Length::Fixed(60.0))
                            .style(text_input_style(colors)),
                        Space::new().width(8),
                        text("1 – 10").size(12).color(colors.text_disabled),
                    ]
                    .align_y(Alignment::Center)
                    .into(),
                    None,
                    true,
                ),
                settings_row(
                    colors,
                    "Speed limit",
                    row![
                        text_input("Unlimited", speed_limit_input)
                            .on_input(Message::SetSpeedLimit)
                            .padding([6, 10])
                            .size(14)
                            .width(Length::Fixed(100.0))
                            .style(text_input_style(colors)),
                        Space::new().width(6),
                        text("KB/s").size(13).color(colors.text_disabled),
                        Space::new().width(8),
                        if !speed_limit_input.is_empty() {
                            button(icon(bs::x_lg).size(12))
                                .on_press(Message::ClearSpeedLimit)
                                .padding([4, 6])
                                .style(icon_button(colors))
                        } else {
                            button(text("")).padding(0).style(icon_button(colors))
                        },
                    ]
                    .align_y(Alignment::Center)
                    .into(),
                    Some(speed_status),
                    false,
                ),
            ],
        ))
        // Speed limit note
        .push(
            container(
                row![
                    icon(bs::info_circle).size(13).color(colors.text_disabled),
                    Space::new().width(8),
                    text("Speed limit changes apply after pausing and resuming active downloads.")
                        .size(12)
                        .color(colors.text_disabled),
                ]
                .align_y(Alignment::Center),
            )
            .padding([0, 4]),
        )
        // Schedule
        .push(settings_group(
            colors,
            is_dark,
            "Schedule",
            vec![settings_row(
                colors,
                "Schedule downloads",
                {
                    let enabled = settings.schedule_enabled;
                    let colon_color = if enabled {
                        colors.text_primary
                    } else {
                        colors.text_disabled
                    };
                    let time_input = |placeholder: &'a str,
                                      value: &'a str,
                                      on_input: fn(String) -> Message|
                     -> iced::widget::TextInput<'a, Message> {
                        let mut inp = text_input(placeholder, value)
                            .padding([6, 6])
                            .size(14)
                            .width(Length::Fixed(36.0))
                            .style(text_input_style(colors));
                        if enabled {
                            inp = inp.on_input(on_input);
                        }
                        inp
                    };
                    row![
                        button(
                            row![
                                icon(if enabled {
                                    bs::toggle_on as fn() -> _
                                } else {
                                    bs::toggle_off as fn() -> _
                                })
                                .size(20),
                                Space::new().width(6),
                                text(if enabled { "On" } else { "Off" }).size(14),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::ToggleSchedule)
                        .padding([6, 14])
                        .style(secondary_button(colors)),
                        Space::new().width(14),
                        time_input("22", sched_from_h, Message::SetScheduleFromH),
                        text(" : ").size(14).color(colon_color),
                        time_input("00", sched_from_m, Message::SetScheduleFromM),
                        Space::new().width(12),
                        text("–").size(17).color(colors.text_primary),
                        Space::new().width(12),
                        time_input("06", sched_to_h, Message::SetScheduleToH),
                        text(" : ").size(14).color(colon_color),
                        time_input("00", sched_to_m, Message::SetScheduleToM),
                    ]
                    .align_y(Alignment::Center)
                    .into()
                },
                None,
                false,
            )],
        ))
        // About
        .push(
            container(
                row![
                    icon_sized(bs::lightning_charge_fill, 15.0).color(colors.text_disabled),
                    Space::new().width(6),
                    text("Bolt v0.1.0").size(12).color(colors.text_disabled),
                    Space::new().width(8),
                    text("·").size(12).color(colors.text_disabled),
                    Space::new().width(8),
                    text("Multi-threaded download manager")
                        .size(12)
                        .color(colors.text_disabled),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 4]),
        );

    scrollable(sections)
        .height(Length::Fill)
        .style(scrollable_style(colors))
        .into()
}

#[allow(dead_code, clippy::too_many_arguments)]
fn build_proxy_settings<'a>(
    colors: ColorScheme,
    is_dark: bool,
    settings: &'a AppSettings,
    proxy_host: &'a str,
    proxy_port: &'a str,
    proxy_user: &'a str,
    proxy_pass: &'a str,
    proxy_testing: bool,
    proxy_test_result: Option<&'a Result<String, String>>,
) -> Element<'a, Message> {
    let active_type = settings.proxy.proxy_type;
    let has_proxy = active_type != ProxyType::None;

    let type_selector = {
        let mut r = Row::new().spacing(4).align_y(Alignment::Center);
        for pt in ProxyType::ALL {
            let is_selected = pt == active_type;
            let btn = button(text(pt.label()).size(13).color(if is_selected {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                colors.text_secondary
            }))
            .on_press(Message::SetProxyType(pt))
            .padding([5, 12])
            .style(filter_button(colors, is_selected));
            r = r.push(btn);
        }
        r
    };

    let status_text = if settings.proxy.is_active() {
        format!(
            "{} via {}:{}",
            active_type.label(),
            proxy_host,
            if proxy_port.is_empty() {
                "—"
            } else {
                proxy_port
            }
        )
    } else if has_proxy {
        "Host required".to_string()
    } else {
        "Direct connection".to_string()
    };

    let mut rows: Vec<Element<'a, Message>> = vec![settings_row(
        colors,
        "Proxy type",
        type_selector.into(),
        Some(status_text),
        true,
    )];

    if has_proxy {
        rows.push(settings_row(
            colors,
            "Server",
            row![
                text_input("Host", proxy_host)
                    .on_input(Message::SetProxyHost)
                    .padding([6, 10])
                    .size(14)
                    .width(Length::Fixed(150.0))
                    .style(text_input_style(colors)),
                Space::new().width(6),
                text(":").size(14).color(colors.text_disabled),
                Space::new().width(6),
                text_input("Port", proxy_port)
                    .on_input(Message::SetProxyPort)
                    .padding([6, 10])
                    .size(14)
                    .width(Length::Fixed(70.0))
                    .style(text_input_style(colors)),
            ]
            .align_y(Alignment::Center)
            .into(),
            None,
            true,
        ));

        rows.push(settings_row(
            colors,
            "Auth",
            row![
                text_input("Username", proxy_user)
                    .on_input(Message::SetProxyUser)
                    .padding([6, 10])
                    .size(14)
                    .width(Length::Fixed(120.0))
                    .style(text_input_style(colors)),
                Space::new().width(8),
                text_input("Password", proxy_pass)
                    .on_input(Message::SetProxyPass)
                    .padding([6, 10])
                    .size(14)
                    .width(Length::Fixed(120.0))
                    .style(text_input_style(colors))
                    .secure(true),
            ]
            .align_y(Alignment::Center)
            .into(),
            Some("Optional".to_string()),
            true,
        ));

        let test_btn: Element<'a, Message> = if proxy_testing {
            button(
                row![
                    icon(bs::arrow_repeat).size(15),
                    Space::new().width(5),
                    text("Testing...").size(14),
                ]
                .align_y(Alignment::Center),
            )
            .padding([5, 12])
            .style(secondary_button(colors))
            .into()
        } else {
            button(
                row![
                    icon(bs::wifi).size(15),
                    Space::new().width(5),
                    text("Test Connection").size(14),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(Message::TestProxy)
            .padding([5, 12])
            .style(secondary_button(colors))
            .into()
        };

        let test_row: Element<'a, Message> = if let Some(result) = proxy_test_result {
            let (result_icon, result_color, result_text) = match result {
                Ok(info) => (
                    bs::check_circle_fill as fn() -> _,
                    Color::from_rgb(0.2, 0.78, 0.4),
                    format!("Connected — {}", info),
                ),
                Err(err) => {
                    let short = err.rsplit(": ").next().unwrap_or(err).to_string();
                    (
                        bs::x_circle_fill as fn() -> _,
                        Color::from_rgb(0.9, 0.3, 0.3),
                        short,
                    )
                }
            };
            row![
                icon(result_icon).size(14).color(result_color),
                Space::new().width(5),
                text(result_text).size(13).color(result_color),
                Space::new().width(Length::Fill),
                test_btn,
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
        } else {
            row![Space::new().width(Length::Fill), test_btn,]
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .into()
        };

        rows.push(
            container(test_row)
                .width(Length::Fill)
                .padding([10, 16])
                .into(),
        );
    }

    settings_group(colors, is_dark, "Network", rows)
}

fn settings_group<'a>(
    colors: ColorScheme,
    is_dark: bool,
    title: &'a str,
    rows: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut col = Column::new().spacing(0).width(Length::Fill);

    col = col.push(
        container(text(title).size(12).color(colors.text_disabled)).padding(iced::Padding {
            top: 0.0,
            right: 4.0,
            bottom: 6.0,
            left: 4.0,
        }),
    );

    let mut card_col = Column::new().spacing(0).width(Length::Fill);

    for (i, row_el) in rows.into_iter().enumerate() {
        if i > 0 {
            card_col = card_col.push(settings_divider(colors));
        }
        card_col = card_col.push(row_el);
    }

    col = col.push(
        container(card_col)
            .width(Length::Fill)
            .style(card_style(colors, is_dark)),
    );

    col.into()
}

fn settings_row<'a>(
    colors: ColorScheme,
    label: &'a str,
    control: Element<'a, Message>,
    description: Option<String>,
    _has_border_bottom: bool,
) -> Element<'a, Message> {
    let label_col: Element<'a, Message> = if let Some(desc) = description {
        column![
            text(label).size(14).color(colors.text_primary),
            text(desc).size(12).color(colors.text_disabled),
        ]
        .spacing(2)
        .width(Length::Fill)
        .into()
    } else {
        text(label)
            .size(14)
            .width(Length::Fill)
            .color(colors.text_primary)
            .into()
    };

    container(
        row![label_col, control]
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([10, 16])
    .into()
}

fn settings_divider(colors: ColorScheme) -> Element<'static, Message> {
    container(Space::new().height(0))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .padding([0, 12])
        .style(move |_theme: &Theme| container::Style {
            text_color: None,
            background: Some(Background::Color(colors.border_light)),
            border: Border::default(),
            shadow: Default::default(),
            ..Default::default()
        })
        .into()
}

fn build_search_bar<'a>(search_query: &'a str, colors: ColorScheme) -> Element<'a, Message> {
    container(
        text_input("Search...", search_query)
            .on_input(Message::SearchChanged)
            .padding([6, 10])
            .size(13)
            .width(Length::Fixed(180.0))
            .icon(text_input::Icon {
                font: iced_fonts::BOOTSTRAP_FONT,
                code_point: '\u{F52A}',
                size: Some(iced::Pixels(12.0)),
                spacing: 8.0,
                side: text_input::Side::Left,
            })
            .style(text_input_style(colors)),
    )
    .padding([4, 8])
    .into()
}

fn build_url_bar<'a>(url_input: &str, colors: ColorScheme, adding: bool) -> Element<'a, Message> {
    let has_url = !url_input.trim().is_empty();
    let is_multi = url_input.contains('\n');

    let input = if adding {
        text_input("Adding download...", url_input)
            .padding([10, 14])
            .size(15)
            .style(text_input_style(colors))
    } else {
        text_input("Paste URL(s) – one per line for batch", url_input)
            .on_input(Message::UrlInputChanged)
            .on_submit(Message::AddDownload)
            .padding([10, 14])
            .size(15)
            .style(text_input_style(colors))
    };

    let add_label = if adding {
        "Adding..."
    } else if is_multi {
        "Add All"
    } else {
        "Add"
    };

    let add_button = if adding {
        button(
            row![
                icon(bs::arrow_repeat).color(Color::from_rgb(0.1, 0.1, 0.1)),
                Space::new().width(6),
                text(add_label).size(15),
            ]
            .align_y(Alignment::Center),
        )
        .padding([10, 20])
        .style(primary_button(colors))
    } else if has_url {
        button(
            row![
                icon(bs::download).color(Color::from_rgb(0.1, 0.1, 0.1)),
                Space::new().width(6),
                text(add_label).size(15),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::AddDownload)
        .padding([10, 20])
        .style(primary_button(colors))
    } else {
        button(
            row![
                icon(bs::download).color(colors.text_disabled),
                Space::new().width(6),
                text(add_label).size(15),
            ]
            .align_y(Alignment::Center),
        )
        .padding([10, 20])
        .style(primary_button(colors))
    };

    let import_button = button(
        row![
            icon(bs::file_earmark_arrow_up).size(15),
            Space::new().width(4),
            text("Import").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::ImportFile)
    .padding([10, 14])
    .style(secondary_button(colors));

    container(
        row![
            icon(bs::link).color(if adding {
                colors.accent_primary
            } else {
                colors.text_secondary
            }),
            Space::new().width(8),
            input,
            Space::new().width(10),
            add_button,
            Space::new().width(6),
            import_button,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([10, 16])
    .style(panel_style(colors))
    .into()
}

fn build_filter_bar(
    active_filter: DownloadFilter,
    colors: ColorScheme,
    counts: (usize, usize, usize, usize, usize),
) -> Element<'static, Message> {
    let (total, active, completed, paused, failed) = counts;

    let filters = [
        (DownloadFilter::All, format!("All ({})", total)),
        (DownloadFilter::Active, format!("Active ({})", active)),
        (DownloadFilter::Completed, format!("Done ({})", completed)),
        (DownloadFilter::Paused, format!("Paused ({})", paused)),
        (DownloadFilter::Failed, format!("Failed ({})", failed)),
    ];

    let mut filter_row = Row::new().spacing(6).align_y(Alignment::Center);

    for (filter, label) in filters {
        let is_active = active_filter == filter;
        filter_row = filter_row.push(
            button(text(label).size(13))
                .on_press(Message::FilterChanged(filter))
                .padding([6, 14])
                .style(filter_button(colors, is_active)),
        );
    }

    filter_row = filter_row.push(Space::new().width(Length::Fill));

    if counts.2 > 0 {
        filter_row = filter_row.push(
            button(
                row![
                    icon(bs::trash).size(13),
                    Space::new().width(4),
                    text("Clear Done").size(13),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(Message::ClearCompleted)
            .padding([6, 12])
            .style(secondary_button(colors)),
        );
    }

    container(filter_row.width(Length::Fill))
        .width(Length::Fill)
        .padding([8, 4])
        .into()
}

fn build_download_list<'a>(
    downloads: &[&'a DownloadItem],
    selected: Option<uuid::Uuid>,
    colors: ColorScheme,
    is_dark: bool,
) -> Element<'a, Message> {
    if downloads.is_empty() {
        let empty = container(
            column![
                icon_sized(bs::cloud_arrow_down, 52.0).color(colors.text_disabled),
                Space::new().height(16),
                text("No downloads yet")
                    .size(17)
                    .color(colors.text_secondary),
                Space::new().height(6),
                text("Paste a URL above to start downloading")
                    .size(14)
                    .color(colors.text_disabled),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(40);

        return empty.into();
    }

    let mut list = Column::new().spacing(8).width(Length::Fill).padding([4, 0]);

    for download in downloads {
        let is_selected = selected == Some(download.id);
        list = list.push(build_download_card(download, is_selected, colors, is_dark));
    }

    scrollable(list)
        .height(Length::Fill)
        .style(scrollable_style(colors))
        .into()
}

fn build_download_card<'a>(
    download: &'a DownloadItem,
    is_selected: bool,
    colors: ColorScheme,
    is_dark: bool,
) -> Element<'a, Message> {
    let id = download.id;

    let category_icon: fn() -> iced::widget::Text<'static> = match download.category {
        FileCategory::Video => bs::camera_video,
        FileCategory::Audio => bs::music_note_beamed,
        FileCategory::Document => bs::file_text,
        FileCategory::Archive => bs::file_zip,
        FileCategory::Image => bs::image,
        FileCategory::Application => bs::terminal,
        FileCategory::Other => bs::file_earmark,
    };

    let status_color = match download.status {
        DownloadStatus::Downloading | DownloadStatus::Connecting => colors.info,
        DownloadStatus::Completed => colors.success,
        DownloadStatus::Paused => colors.warning,
        DownloadStatus::Failed | DownloadStatus::Cancelled => colors.error,
        DownloadStatus::Queued => colors.text_secondary,
    };

    let display_name = truncate_filename(&download.filename, 65);

    let status_text = download.status.label().to_string();

    let name_and_status = row![
        icon(category_icon).color(colors.accent_primary),
        Space::new().width(8),
        column![text(display_name).size(14).color(colors.text_primary),].width(Length::Fill),
        Space::new().width(8),
        container(text(status_text).size(11))
            .padding([2, 8])
            .style(status_badge_style(status_color)),
        Space::new().width(4),
        build_action_buttons(download, colors),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let progress_percent = download.progress_percent();

    let status = download.status;
    let progress = progress_bar(0.0..=100.0, progress_percent)
        .girth(4.0)
        .style(move |theme| match status {
            DownloadStatus::Completed => (progress_complete_style(colors))(theme),
            DownloadStatus::Paused => (progress_paused_style(colors))(theme),
            DownloadStatus::Failed | DownloadStatus::Cancelled => {
                (progress_error_style(colors))(theme)
            }
            _ => (progress_style(colors))(theme),
        });

    let size_text = match download.total_size {
        Some(total) => format!(
            "{} / {}",
            format_bytes(download.downloaded),
            format_bytes(total)
        ),
        None => format_bytes(download.downloaded),
    };

    let mut info = Row::new().spacing(12).align_y(Alignment::Center);

    info = info.push(
        text(size_text)
            .size(12)
            .font(JETBRAINS_MONO)
            .color(colors.text_secondary),
    );

    if download.total_size.is_some() {
        info = info.push(
            text(format!("{:.1}%", progress_percent))
                .size(12)
                .font(JETBRAINS_MONO)
                .color(colors.accent_primary),
        );
    }

    info = info.push(Space::new().width(Length::Fill));

    if download.status == DownloadStatus::Downloading {
        info = info.push(
            text(format_speed(download.speed))
                .size(12)
                .font(JETBRAINS_MONO)
                .color(colors.info),
        );
        if let Some(eta) = download.eta_seconds() {
            info = info.push(
                text(format_eta(eta))
                    .size(12)
                    .font(JETBRAINS_MONO)
                    .color(colors.text_disabled),
            );
        }
    }

    let mut card_content = Column::new()
        .spacing(5)
        .width(Length::Fill)
        .push(name_and_status)
        .push(progress)
        .push(info.width(Length::Fill));

    if let Some(ref error) = download.error {
        card_content = card_content.push(
            text(format!("Error: {}", error))
                .size(12)
                .color(colors.error),
        );
    }

    let card = button(
        container(card_content)
            .padding([10, 14])
            .width(Length::Fill),
    )
    .on_press(Message::SelectDownload(Some(id)))
    .width(Length::Fill)
    .style(transparent_button(colors, is_selected));

    container(card)
        .width(Length::Fill)
        .style(download_card_style(colors, is_dark, is_selected))
        .into()
}

fn build_action_buttons(download: &DownloadItem, colors: ColorScheme) -> Element<'_, Message> {
    let id = download.id;

    let mut actions = Row::new().spacing(4).align_y(Alignment::Center);

    match download.status {
        DownloadStatus::Downloading | DownloadStatus::Connecting => {
            actions = actions.push(
                button(icon(bs::pause_fill))
                    .on_press(Message::PauseDownload(id))
                    .padding([6, 8])
                    .style(icon_button(colors)),
            );
            actions = actions.push(
                button(icon(bs::x_lg))
                    .on_press(Message::CancelDownload(id))
                    .padding([6, 8])
                    .style(danger_button(colors)),
            );
        }
        DownloadStatus::Paused => {
            actions = actions.push(
                button(icon(bs::play_fill))
                    .on_press(Message::ResumeDownload(id))
                    .padding([6, 8])
                    .style(icon_button(colors)),
            );
            actions = actions.push(
                button(icon(bs::x_lg))
                    .on_press(Message::CancelDownload(id))
                    .padding([6, 8])
                    .style(danger_button(colors)),
            );
        }
        DownloadStatus::Completed => {
            actions = actions.push(
                button(icon(bs::folder_symlink))
                    .on_press(Message::OpenFolder(id))
                    .padding([6, 8])
                    .style(icon_button(colors)),
            );
            actions = actions.push(
                button(icon(bs::trash))
                    .on_press(Message::RemoveDownload(id))
                    .padding([6, 8])
                    .style(danger_button(colors)),
            );
        }
        DownloadStatus::Failed | DownloadStatus::Cancelled => {
            actions = actions.push(
                button(icon(bs::arrow_repeat))
                    .on_press(Message::RetryDownload(id))
                    .padding([6, 8])
                    .style(icon_button(colors)),
            );
            actions = actions.push(
                button(icon(bs::trash))
                    .on_press(Message::RemoveDownload(id))
                    .padding([6, 8])
                    .style(danger_button(colors)),
            );
        }
        DownloadStatus::Queued => {
            actions = actions.push(
                button(icon(bs::play_fill))
                    .on_press(Message::StartDownload(id))
                    .padding([6, 8])
                    .style(icon_button(colors)),
            );
            actions = actions.push(
                button(icon(bs::x_lg))
                    .on_press(Message::RemoveDownload(id))
                    .padding([6, 8])
                    .style(danger_button(colors)),
            );
        }
    }

    actions.into()
}

fn build_status_bar(
    total_speed: f64,
    counts: (usize, usize, usize, usize, usize),
    colors: ColorScheme,
    download_dir: &std::path::Path,
    network_online: bool,
) -> Element<'static, Message> {
    let (total, active, completed, _paused, _failed) = counts;

    let dir_display: String = download_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Downloads")
        .to_string();

    let net_icon: fn() -> iced::widget::Text<'static> = if network_online {
        bs::wifi
    } else {
        bs::wifi_off
    };
    let net_color = if network_online {
        colors.success
    } else {
        colors.warning
    };

    container(
        row![
            icon(bs::folder).size(13).color(colors.text_disabled),
            Space::new().width(4),
            button(text(dir_display).size(12).color(colors.text_secondary))
                .on_press(Message::ChooseDownloadDir)
                .padding([2, 6])
                .style(icon_button(colors)),
            Space::new().width(Length::Fill),
            icon(net_icon).size(12).color(net_color),
            Space::new().width(12),
            text(format!("{} downloads", total))
                .size(13)
                .color(colors.text_disabled),
            Space::new().width(16),
            if active > 0 {
                text(format!("{} active", active))
                    .size(13)
                    .color(colors.info)
            } else {
                text("").size(13)
            },
            Space::new().width(16),
            if completed > 0 {
                text(format!("{} done", completed))
                    .size(13)
                    .color(colors.success)
            } else {
                text("").size(13)
            },
            Space::new().width(16),
            if active > 0 {
                row![
                    icon(bs::speedometer).size(13).color(colors.accent_primary),
                    Space::new().width(4),
                    text(format_speed(total_speed))
                        .size(13)
                        .font(JETBRAINS_MONO)
                        .color(colors.accent_primary),
                ]
                .align_y(Alignment::Center)
            } else {
                row![]
            },
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([8, 16])
    .style(move |_theme: &Theme| container::Style {
        text_color: Some(colors.text_secondary),
        background: Some(Background::Color(colors.surface)),
        border: Border {
            color: colors.border_light,
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 8.0,
                bottom_left: 8.0,
            },
        },
        shadow: Default::default(),
        ..Default::default()
    })
    .into()
}
