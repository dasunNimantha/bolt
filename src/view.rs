use crate::message::Message;
use crate::model::{DownloadFilter, DownloadItem, DownloadStatus, FileCategory};
use crate::theme::{
    get_colors, CardStyle, ColorScheme, DangerButtonStyle, DownloadCardStyle, FilterButtonStyle,
    IconButtonStyle, PanelStyle, PrimaryButtonStyle, ProgressBarCompleteStyle,
    ProgressBarErrorStyle, ProgressBarPausedStyle, ProgressBarStyle, ScrollableStyle,
    SecondaryButtonStyle, StatusBadgeStyle, TextInputStyle, ThemeMode, ToggleStyle,
};
use crate::utils::format::{format_bytes, format_eta, format_speed, truncate_filename};
use iced::widget::{
    button, checkbox, column, container, progress_bar, row, scrollable, text, text_input, Column,
    Row, Space,
};
use iced::{Alignment, Color, Element, Font, Length, Theme};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};

const JETBRAINS_MONO: Font = Font::with_name("JetBrains Mono");

fn icon(bootstrap: Bootstrap) -> iced::widget::Text<'static> {
    icon_to_text(bootstrap).size(16.0)
}

fn icon_sized(bootstrap: Bootstrap, size: f32) -> iced::widget::Text<'static> {
    icon_to_text(bootstrap).size(size)
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
) -> Element<'a, Message> {
    let colors = get_colors(theme_mode);
    let is_dark = theme_mode == ThemeMode::Dark;

    let header = build_header(colors, is_dark);
    let url_bar = build_url_bar(url_input, colors, adding);
    let filter_bar = build_filter_bar(filter, colors, counts);

    let filtered: Vec<&DownloadItem> = downloads
        .iter()
        .filter(|d| filter.matches(d.status))
        .collect();

    let download_list = build_download_list(&filtered, selected, colors, is_dark);
    let status_bar = build_status_bar(total_speed, counts, colors, download_dir);

    let mut content = Column::new()
        .spacing(6)
        .width(Length::Fill)
        .height(Length::Fill)
        .push(header)
        .push(url_bar);

    if adding {
        content = content.push(
            container(
                row![
                    icon(Bootstrap::HourglassSplit)
                        .style(iced::theme::Text::Color(colors.accent_primary)),
                    Space::with_width(10),
                    text("Fetching file info...")
                        .size(13)
                        .style(iced::theme::Text::Color(colors.accent_primary)),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([10, 20])
            .style(iced::theme::Container::Custom(Box::new(
                move |_: &Theme| iced::widget::container::Appearance {
                    text_color: None,
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.95, 0.75, 0.25, 0.08,
                    ))),
                    border: iced::Border {
                        color: colors.accent_primary,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Default::default(),
                },
            ))),
        );
    }

    if let Some(err) = error_message {
        content = content.push(
            container(
                row![
                    icon(Bootstrap::ExclamationTriangleFill)
                        .style(iced::theme::Text::Color(colors.error)),
                    Space::with_width(8),
                    text(err)
                        .size(13)
                        .style(iced::theme::Text::Color(colors.error)),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 20])
            .style(iced::theme::Container::Custom(Box::new(
                move |_: &Theme| iced::widget::container::Appearance {
                    text_color: None,
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.95, 0.35, 0.35, 0.1,
                    ))),
                    border: iced::Border {
                        color: colors.error,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Default::default(),
                },
            ))),
        );
    }

    content = content
        .push(filter_bar)
        .push(download_list)
        .push(status_bar);

    let content: Element<'a, Message> = content.into();

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([12, 16])
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &Theme| iced::widget::container::Appearance {
                text_color: Some(colors.text_primary),
                background: Some(iced::Background::Color(colors.bg_primary)),
                border: iced::Border::default(),
                shadow: Default::default(),
            },
        )))
        .into()
}

fn build_header(colors: ColorScheme, is_dark: bool) -> Element<'static, Message> {
    container(
        row![
            row![
                icon_sized(Bootstrap::LightningChargeFill, 24.0)
                    .style(iced::theme::Text::Color(colors.accent_primary)),
                Space::with_width(10),
                text("Bolt")
                    .size(22)
                    .style(iced::theme::Text::Color(colors.text_primary)),
                Space::with_width(6),
                text("Download Manager")
                    .size(13)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
            ]
            .align_items(Alignment::Center),
            Space::with_width(Length::Fill),
            checkbox(if is_dark { "Dark" } else { "Light" }, !is_dark,)
                .on_toggle(|_| Message::ToggleTheme)
                .size(18)
                .spacing(8)
                .style(iced::theme::Checkbox::Custom(Box::new(ToggleStyle {
                    colors,
                }))),
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([10, 16])
    .style(iced::theme::Container::Custom(Box::new(CardStyle {
        colors,
        is_dark,
    })))
    .into()
}

fn build_url_bar<'a>(url_input: &str, colors: ColorScheme, adding: bool) -> Element<'a, Message> {
    let has_url = !url_input.trim().is_empty();

    let input = if adding {
        text_input("Adding download...", url_input)
            .padding([10, 14])
            .size(14)
            .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle {
                colors,
            })))
    } else {
        text_input("Paste download URL here...", url_input)
            .on_input(Message::UrlInputChanged)
            .on_submit(Message::AddDownload)
            .padding([10, 14])
            .size(14)
            .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle {
                colors,
            })))
    };

    let add_button = if adding {
        button(
            row![
                icon(Bootstrap::ArrowRepeat)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                Space::with_width(6),
                text("Adding...").size(14),
            ]
            .align_items(Alignment::Center),
        )
        .padding([10, 20])
        .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
            colors,
        })))
    } else if has_url {
        button(
            row![
                icon(Bootstrap::Download)
                    .style(iced::theme::Text::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                Space::with_width(6),
                text("Add").size(14),
            ]
            .align_items(Alignment::Center),
        )
        .on_press(Message::AddDownload)
        .padding([10, 20])
        .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
            colors,
        })))
    } else {
        button(
            row![
                icon(Bootstrap::Download).style(iced::theme::Text::Color(colors.text_disabled)),
                Space::with_width(6),
                text("Add").size(14),
            ]
            .align_items(Alignment::Center),
        )
        .padding([10, 20])
        .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
            colors,
        })))
    };

    container(
        row![
            icon(Bootstrap::Link).style(iced::theme::Text::Color(if adding {
                colors.accent_primary
            } else {
                colors.text_secondary
            })),
            Space::with_width(8),
            input,
            Space::with_width(10),
            add_button,
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([10, 16])
    .style(iced::theme::Container::Custom(Box::new(PanelStyle {
        colors,
    })))
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

    let mut filter_row = Row::new().spacing(6).align_items(Alignment::Center);

    for (filter, label) in filters {
        let is_active = active_filter == filter;
        filter_row = filter_row.push(
            button(text(label).size(12))
                .on_press(Message::FilterChanged(filter))
                .padding([6, 14])
                .style(iced::theme::Button::Custom(Box::new(FilterButtonStyle {
                    colors,
                    is_active,
                }))),
        );
    }

    filter_row = filter_row.push(Space::with_width(Length::Fill));

    if counts.2 > 0 {
        filter_row = filter_row.push(
            button(
                row![
                    icon(Bootstrap::Trash).size(12),
                    Space::with_width(4),
                    text("Clear Done").size(12),
                ]
                .align_items(Alignment::Center),
            )
            .on_press(Message::ClearCompleted)
            .padding([6, 12])
            .style(iced::theme::Button::Custom(Box::new(
                SecondaryButtonStyle { colors },
            ))),
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
                icon_sized(Bootstrap::CloudArrowDown, 48.0)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
                Space::with_height(16),
                text("No downloads yet")
                    .size(16)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
                Space::with_height(6),
                text("Paste a URL above to start downloading")
                    .size(13)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
            ]
            .align_items(Alignment::Center)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
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
        .style(iced::theme::Scrollable::Custom(Box::new(ScrollableStyle {
            colors,
        })))
        .into()
}

fn build_download_card<'a>(
    download: &'a DownloadItem,
    is_selected: bool,
    colors: ColorScheme,
    is_dark: bool,
) -> Element<'a, Message> {
    let id = download.id;

    let category_icon = match download.category {
        FileCategory::Video => Bootstrap::CameraVideo,
        FileCategory::Audio => Bootstrap::MusicNoteBeamed,
        FileCategory::Document => Bootstrap::FileText,
        FileCategory::Archive => Bootstrap::FileZip,
        FileCategory::Image => Bootstrap::Image,
        FileCategory::Application => Bootstrap::Terminal,
        FileCategory::Other => Bootstrap::FileEarmark,
    };

    let status_color = match download.status {
        DownloadStatus::Downloading | DownloadStatus::Connecting => colors.info,
        DownloadStatus::Completed => colors.success,
        DownloadStatus::Paused => colors.warning,
        DownloadStatus::Failed | DownloadStatus::Cancelled => colors.error,
        DownloadStatus::Queued => colors.text_secondary,
    };

    let display_name = truncate_filename(&download.filename, 65);

    let name_and_status = row![
        icon(category_icon).style(iced::theme::Text::Color(colors.accent_primary)),
        Space::with_width(8),
        column![text(display_name)
            .size(13)
            .style(iced::theme::Text::Color(colors.text_primary)),]
        .width(Length::Fill),
        Space::with_width(8),
        container(text(download.status.label()).size(10))
            .padding([2, 8])
            .style(iced::theme::Container::Custom(Box::new(StatusBadgeStyle {
                color: status_color,
            }))),
        Space::with_width(4),
        build_action_buttons(download, colors),
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);

    let progress_percent = download.progress_percent();

    let progress =
        progress_bar(0.0..=100.0, progress_percent)
            .height(4)
            .style(match download.status {
                DownloadStatus::Completed => {
                    iced::theme::ProgressBar::Custom(Box::new(ProgressBarCompleteStyle { colors }))
                }
                DownloadStatus::Paused => {
                    iced::theme::ProgressBar::Custom(Box::new(ProgressBarPausedStyle { colors }))
                }
                DownloadStatus::Failed | DownloadStatus::Cancelled => {
                    iced::theme::ProgressBar::Custom(Box::new(ProgressBarErrorStyle { colors }))
                }
                _ => iced::theme::ProgressBar::Custom(Box::new(ProgressBarStyle { colors })),
            });

    let size_text = match download.total_size {
        Some(total) => format!(
            "{} / {}",
            format_bytes(download.downloaded),
            format_bytes(total)
        ),
        None => format_bytes(download.downloaded),
    };

    let mut info = Row::new().spacing(12).align_items(Alignment::Center);

    info = info.push(
        text(&size_text)
            .size(11)
            .font(JETBRAINS_MONO)
            .style(iced::theme::Text::Color(colors.text_secondary)),
    );

    if download.total_size.is_some() {
        info = info.push(
            text(format!("{:.1}%", progress_percent))
                .size(11)
                .font(JETBRAINS_MONO)
                .style(iced::theme::Text::Color(colors.accent_primary)),
        );
    }

    info = info.push(Space::with_width(Length::Fill));

    if download.status == DownloadStatus::Downloading {
        info = info.push(
            text(format_speed(download.speed))
                .size(11)
                .font(JETBRAINS_MONO)
                .style(iced::theme::Text::Color(colors.info)),
        );
        if let Some(eta) = download.eta_seconds() {
            info = info.push(
                text(format_eta(eta))
                    .size(11)
                    .font(JETBRAINS_MONO)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
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
                .size(11)
                .style(iced::theme::Text::Color(colors.error)),
        );
    }

    let card = button(
        container(card_content)
            .padding([10, 14])
            .width(Length::Fill),
    )
    .on_press(Message::SelectDownload(Some(id)))
    .width(Length::Fill)
    .style(iced::theme::Button::Custom(Box::new(
        TransparentButtonStyle {
            colors,
            is_selected,
        },
    )));

    container(card)
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(
            DownloadCardStyle {
                colors,
                is_dark,
                is_selected,
            },
        )))
        .into()
}

fn build_action_buttons(download: &DownloadItem, colors: ColorScheme) -> Element<'_, Message> {
    let id = download.id;

    let mut actions = Row::new().spacing(4).align_items(Alignment::Center);

    match download.status {
        DownloadStatus::Downloading | DownloadStatus::Connecting => {
            actions = actions.push(
                button(icon(Bootstrap::PauseFill))
                    .on_press(Message::PauseDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(IconButtonStyle {
                        colors,
                    }))),
            );
            actions = actions.push(
                button(icon(Bootstrap::XLg))
                    .on_press(Message::CancelDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
                        colors,
                    }))),
            );
        }
        DownloadStatus::Paused => {
            actions = actions.push(
                button(icon(Bootstrap::PlayFill))
                    .on_press(Message::ResumeDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(IconButtonStyle {
                        colors,
                    }))),
            );
            actions = actions.push(
                button(icon(Bootstrap::XLg))
                    .on_press(Message::CancelDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
                        colors,
                    }))),
            );
        }
        DownloadStatus::Completed => {
            actions = actions.push(
                button(icon(Bootstrap::FolderSymlink))
                    .on_press(Message::OpenFolder(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(IconButtonStyle {
                        colors,
                    }))),
            );
            actions = actions.push(
                button(icon(Bootstrap::Trash))
                    .on_press(Message::RemoveDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
                        colors,
                    }))),
            );
        }
        DownloadStatus::Failed | DownloadStatus::Cancelled => {
            actions = actions.push(
                button(icon(Bootstrap::ArrowRepeat))
                    .on_press(Message::RetryDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(IconButtonStyle {
                        colors,
                    }))),
            );
            actions = actions.push(
                button(icon(Bootstrap::Trash))
                    .on_press(Message::RemoveDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
                        colors,
                    }))),
            );
        }
        DownloadStatus::Queued => {
            actions = actions.push(
                button(icon(Bootstrap::PlayFill))
                    .on_press(Message::StartDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(IconButtonStyle {
                        colors,
                    }))),
            );
            actions = actions.push(
                button(icon(Bootstrap::XLg))
                    .on_press(Message::RemoveDownload(id))
                    .padding([6, 8])
                    .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
                        colors,
                    }))),
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
) -> Element<'static, Message> {
    let (total, active, completed, _paused, _failed) = counts;

    let dir_display = download_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Downloads");

    container(
        row![
            icon(Bootstrap::Folder)
                .size(12)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_width(4),
            button(
                text(dir_display)
                    .size(11)
                    .style(iced::theme::Text::Color(colors.text_secondary))
            )
            .on_press(Message::ChooseDownloadDir)
            .padding([2, 6])
            .style(iced::theme::Button::Custom(Box::new(IconButtonStyle {
                colors,
            }))),
            Space::with_width(Length::Fill),
            text(format!("{} downloads", total))
                .size(12)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_width(16),
            if active > 0 {
                text(format!("{} active", active))
                    .size(12)
                    .style(iced::theme::Text::Color(colors.info))
            } else {
                text("").size(12)
            },
            Space::with_width(16),
            if completed > 0 {
                text(format!("{} done", completed))
                    .size(12)
                    .style(iced::theme::Text::Color(colors.success))
            } else {
                text("").size(12)
            },
            Space::with_width(16),
            if active > 0 {
                row![
                    icon(Bootstrap::Speedometer)
                        .size(12)
                        .style(iced::theme::Text::Color(colors.accent_primary)),
                    Space::with_width(4),
                    text(format_speed(total_speed))
                        .size(12)
                        .font(JETBRAINS_MONO)
                        .style(iced::theme::Text::Color(colors.accent_primary)),
                ]
                .align_items(Alignment::Center)
            } else {
                row![]
            },
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([8, 16])
    .style(iced::theme::Container::Custom(Box::new(
        move |_: &Theme| iced::widget::container::Appearance {
            text_color: Some(colors.text_secondary),
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border_light,
                width: 1.0,
                radius: [0.0, 0.0, 8.0, 8.0].into(),
            },
            shadow: Default::default(),
        },
    )))
    .into()
}

// Transparent button for clickable download cards
struct TransparentButtonStyle {
    colors: ColorScheme,
    is_selected: bool,
}

impl iced::widget::button::StyleSheet for TransparentButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            text_color: self.colors.text_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(if self.is_selected {
                Color::TRANSPARENT
            } else {
                Color::from_rgba(0.5, 0.5, 0.5, 0.05)
            })),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            text_color: self.colors.text_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.hovered(style)
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }
}
