use iced::theme::{self, Theme};
use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub struct ColorScheme {
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub accent_hover: Color,
    pub accent_dark: Color,

    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    pub bg_hover: Color,

    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub surface_elevated: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    pub border: Color,
    pub border_light: Color,
    pub border_focus: Color,
}

impl ColorScheme {
    pub fn dark() -> Self {
        Self {
            accent_primary: Color::from_rgb(0.95, 0.75, 0.25),
            accent_secondary: Color::from_rgb(0.85, 0.65, 0.20),
            accent_hover: Color::from_rgb(1.0, 0.85, 0.35),
            accent_dark: Color::from_rgb(0.75, 0.55, 0.15),

            bg_primary: Color::from_rgb(0.08, 0.08, 0.10),
            bg_secondary: Color::from_rgb(0.12, 0.12, 0.14),
            bg_tertiary: Color::from_rgb(0.16, 0.16, 0.18),
            bg_hover: Color::from_rgb(0.18, 0.18, 0.20),

            surface: Color::from_rgb(0.11, 0.11, 0.13),
            surface_hover: Color::from_rgb(0.15, 0.15, 0.17),
            surface_active: Color::from_rgb(0.19, 0.19, 0.21),
            surface_elevated: Color::from_rgb(0.13, 0.13, 0.15),

            text_primary: Color::from_rgb(0.95, 0.95, 0.97),
            text_secondary: Color::from_rgb(0.65, 0.65, 0.70),
            text_disabled: Color::from_rgb(0.45, 0.45, 0.50),

            success: Color::from_rgb(0.25, 0.85, 0.45),
            warning: Color::from_rgb(1.0, 0.75, 0.25),
            error: Color::from_rgb(0.95, 0.35, 0.35),
            info: Color::from_rgb(0.35, 0.65, 0.95),

            border: Color::from_rgb(0.22, 0.22, 0.26),
            border_light: Color::from_rgb(0.18, 0.18, 0.22),
            border_focus: Color::from_rgb(0.95, 0.75, 0.25),
        }
    }

    pub fn light() -> Self {
        Self {
            accent_primary: Color::from_rgb(0.85, 0.60, 0.10),
            accent_secondary: Color::from_rgb(0.75, 0.50, 0.05),
            accent_hover: Color::from_rgb(0.95, 0.70, 0.20),
            accent_dark: Color::from_rgb(0.65, 0.45, 0.00),

            bg_primary: Color::from_rgb(0.97, 0.97, 0.98),
            bg_secondary: Color::from_rgb(0.94, 0.94, 0.96),
            bg_tertiary: Color::from_rgb(0.91, 0.91, 0.93),
            bg_hover: Color::from_rgb(0.89, 0.89, 0.91),

            surface: Color::from_rgb(1.0, 1.0, 1.0),
            surface_hover: Color::from_rgb(0.98, 0.98, 1.0),
            surface_active: Color::from_rgb(0.95, 0.95, 0.97),
            surface_elevated: Color::from_rgb(1.0, 1.0, 1.0),

            text_primary: Color::from_rgb(0.10, 0.10, 0.12),
            text_secondary: Color::from_rgb(0.40, 0.40, 0.45),
            text_disabled: Color::from_rgb(0.60, 0.60, 0.65),

            success: Color::from_rgb(0.20, 0.75, 0.40),
            warning: Color::from_rgb(0.90, 0.65, 0.15),
            error: Color::from_rgb(0.90, 0.30, 0.30),
            info: Color::from_rgb(0.30, 0.55, 0.90),

            border: Color::from_rgb(0.82, 0.82, 0.87),
            border_light: Color::from_rgb(0.88, 0.88, 0.92),
            border_focus: Color::from_rgb(0.85, 0.60, 0.10),
        }
    }
}

pub fn get_colors(mode: ThemeMode) -> ColorScheme {
    match mode {
        ThemeMode::Dark => ColorScheme::dark(),
        ThemeMode::Light => ColorScheme::light(),
    }
}

pub fn bolt_theme(mode: ThemeMode) -> Theme {
    let colors = get_colors(mode);
    Theme::custom(
        "Bolt".to_string(),
        theme::Palette {
            background: colors.bg_primary,
            text: colors.text_primary,
            primary: colors.accent_primary,
            success: colors.success,
            danger: colors.error,
        },
    )
}

// ============== Container Styles ==============

pub struct CardStyle {
    pub mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for CardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_primary),
            background: Some(iced::Background::Color(colors.surface_elevated)),
            border: iced::Border {
                color: colors.border_light,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: if self.mode == ThemeMode::Dark {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                },
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        }
    }
}

pub struct PanelStyle {
    pub mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for PanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_primary),
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

pub struct DownloadCardStyle {
    pub mode: ThemeMode,
    pub is_selected: bool,
}

impl iced::widget::container::StyleSheet for DownloadCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        let bg_color = if self.is_selected {
            if self.mode == ThemeMode::Dark {
                Color::from_rgba(0.95, 0.75, 0.25, 0.12)
            } else {
                Color::from_rgba(0.85, 0.60, 0.10, 0.12)
            }
        } else {
            colors.surface
        };

        iced::widget::container::Appearance {
            text_color: Some(colors.text_primary),
            background: Some(iced::Background::Color(bg_color)),
            border: iced::Border {
                color: if self.is_selected {
                    colors.accent_primary
                } else {
                    colors.border_light
                },
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: iced::Shadow {
                color: if self.mode == ThemeMode::Dark {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.1)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.05)
                },
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
        }
    }
}

pub struct StatusBadgeStyle {
    pub color: Color,
}

impl iced::widget::container::StyleSheet for StatusBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(iced::Background::Color(self.color)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

// ============== Button Styles ==============

pub struct PrimaryButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for PrimaryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.accent_primary)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = Some(iced::Background::Color(colors.accent_hover));
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = Some(iced::Background::Color(colors.accent_dark));
        appearance
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = Some(iced::Background::Color(colors.bg_tertiary));
        appearance.text_color = colors.text_disabled;
        appearance
    }
}

pub struct SecondaryButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for SecondaryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: colors.text_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = Some(iced::Background::Color(colors.surface_hover));
        appearance.border.color = colors.accent_primary;
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = Some(iced::Background::Color(colors.surface_active));
        appearance
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = Some(iced::Background::Color(colors.bg_tertiary));
        appearance.text_color = colors.text_disabled;
        appearance
    }
}

pub struct DangerButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for DangerButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: colors.text_secondary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let error_color = Color::from_rgba(0.85, 0.25, 0.25, 1.0);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(error_color)),
            border: iced::Border {
                color: error_color,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let error_color = Color::from_rgba(0.75, 0.20, 0.20, 1.0);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(error_color)),
            border: iced::Border {
                color: error_color,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.bg_tertiary)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: colors.text_disabled,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

pub struct IconButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for IconButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            text_color: colors.text_secondary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.surface_hover)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            text_color: colors.accent_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.surface_active)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            text_color: colors.accent_dark,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border::default(),
            text_color: colors.text_disabled,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

pub struct FilterButtonStyle {
    pub mode: ThemeMode,
    pub is_active: bool,
}

impl iced::widget::button::StyleSheet for FilterButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        if self.is_active {
            iced::widget::button::Appearance {
                background: Some(iced::Background::Color(colors.accent_primary)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.1, 0.1, 0.1),
                shadow: Default::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        } else {
            iced::widget::button::Appearance {
                background: Some(iced::Background::Color(colors.bg_tertiary)),
                border: iced::Border {
                    color: colors.border_light,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: colors.text_secondary,
                shadow: Default::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        if self.is_active {
            iced::widget::button::Appearance {
                background: Some(iced::Background::Color(colors.accent_hover)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                text_color: Color::from_rgb(0.1, 0.1, 0.1),
                shadow: Default::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        } else {
            iced::widget::button::Appearance {
                background: Some(iced::Background::Color(colors.bg_hover)),
                border: iced::Border {
                    color: colors.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                text_color: colors.text_primary,
                shadow: Default::default(),
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.hovered(style)
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }
}

// ============== Input Styles ==============

pub struct TextInputStyle {
    pub mode: ThemeMode,
}

impl iced::widget::text_input::StyleSheet for TextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(colors.bg_secondary),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon_color: colors.text_secondary,
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(colors.bg_secondary),
            border: iced::Border {
                color: colors.border_focus,
                width: 2.0,
                radius: 8.0.into(),
            },
            icon_color: colors.accent_primary,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        get_colors(self.mode).text_disabled
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        get_colors(self.mode).text_primary
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.95, 0.75, 0.25, 0.3)
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        let mut appearance = self.active(style);
        let colors = get_colors(self.mode);
        appearance.background = iced::Background::Color(colors.bg_tertiary);
        appearance
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        get_colors(self.mode).text_disabled
    }
}

// ============== Progress Bar Style ==============

pub struct ProgressBarStyle {
    pub mode: ThemeMode,
}

impl iced::widget::progress_bar::StyleSheet for ProgressBarStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::progress_bar::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::progress_bar::Appearance {
            background: iced::Background::Color(colors.bg_tertiary),
            bar: iced::Background::Color(colors.accent_primary),
            border_radius: 4.0.into(),
        }
    }
}

pub struct ProgressBarPausedStyle {
    pub mode: ThemeMode,
}

impl iced::widget::progress_bar::StyleSheet for ProgressBarPausedStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::progress_bar::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::progress_bar::Appearance {
            background: iced::Background::Color(colors.bg_tertiary),
            bar: iced::Background::Color(colors.warning),
            border_radius: 4.0.into(),
        }
    }
}

pub struct ProgressBarCompleteStyle {
    pub mode: ThemeMode,
}

impl iced::widget::progress_bar::StyleSheet for ProgressBarCompleteStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::progress_bar::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::progress_bar::Appearance {
            background: iced::Background::Color(colors.bg_tertiary),
            bar: iced::Background::Color(colors.success),
            border_radius: 4.0.into(),
        }
    }
}

pub struct ProgressBarErrorStyle {
    pub mode: ThemeMode,
}

impl iced::widget::progress_bar::StyleSheet for ProgressBarErrorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::progress_bar::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::progress_bar::Appearance {
            background: iced::Background::Color(colors.bg_tertiary),
            bar: iced::Background::Color(colors.error),
            border_radius: 4.0.into(),
        }
    }
}

// ============== Scrollable Style ==============

pub struct ScrollableStyle {
    pub mode: ThemeMode,
}

impl iced::widget::scrollable::StyleSheet for ScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::scrollable::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                border: iced::Border::default(),
                scroller: iced::widget::scrollable::Scroller {
                    color: colors.bg_tertiary,
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        _is_mouse_over_scrollbar: bool,
    ) -> iced::widget::scrollable::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::scrollable::Appearance {
            container: iced::widget::container::Appearance::default(),
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                border: iced::Border::default(),
                scroller: iced::widget::scrollable::Scroller {
                    color: colors.border,
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 4.0.into(),
                    },
                },
            },
            gap: None,
        }
    }
}

// ============== Toggle/Checkbox Style ==============

pub struct ToggleStyle {
    pub mode: ThemeMode,
}

impl iced::widget::checkbox::StyleSheet for ToggleStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::checkbox::Appearance {
            background: if is_checked {
                iced::Background::Color(colors.accent_primary)
            } else {
                iced::Background::Color(colors.bg_tertiary)
            },
            icon_color: if is_checked {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                Color::WHITE
            },
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            text_color: Some(colors.text_primary),
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_checked: bool,
    ) -> iced::widget::checkbox::Appearance {
        let mut appearance = self.active(style, is_checked);
        let colors = get_colors(self.mode);
        if is_checked {
            appearance.background = iced::Background::Color(colors.accent_hover);
        } else {
            appearance.background = iced::Background::Color(colors.bg_hover);
        }
        appearance
    }
}
