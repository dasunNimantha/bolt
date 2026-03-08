use iced::widget::{button, checkbox, container, progress_bar, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemeMode {
    pub fn effective(self, system_is_dark: bool) -> ThemeMode {
        match self {
            ThemeMode::System => {
                if system_is_dark {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                }
            }
            other => other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
            ThemeMode::System => "System",
        }
    }
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
        ThemeMode::Dark | ThemeMode::System => ColorScheme::dark(),
        ThemeMode::Light => ColorScheme::light(),
    }
}

pub fn bolt_theme(mode: ThemeMode) -> Theme {
    let colors = get_colors(mode);
    Theme::custom(
        "Bolt".to_string(),
        iced::theme::Palette {
            background: colors.bg_primary,
            text: colors.text_primary,
            primary: colors.accent_primary,
            success: colors.success,
            danger: colors.error,
            warning: colors.warning,
        },
    )
}

// ============== Container Styles ==============

pub fn card_style(colors: ColorScheme, is_dark: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        text_color: Some(colors.text_primary),
        background: Some(Background::Color(colors.surface_elevated)),
        border: Border {
            color: colors.border_light,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: if is_dark {
                Color::from_rgba(0.0, 0.0, 0.0, 0.15)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.08)
            },
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

pub fn panel_style(colors: ColorScheme) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        text_color: Some(colors.text_primary),
        background: Some(Background::Color(colors.surface)),
        border: Border {
            color: colors.border,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Default::default(),
        ..Default::default()
    }
}

pub fn download_card_style(
    colors: ColorScheme,
    is_dark: bool,
    is_selected: bool,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let bg_color = if is_selected {
            if is_dark {
                Color::from_rgba(0.95, 0.75, 0.25, 0.12)
            } else {
                Color::from_rgba(0.85, 0.60, 0.10, 0.12)
            }
        } else {
            colors.surface
        };

        container::Style {
            text_color: Some(colors.text_primary),
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: if is_selected {
                    colors.accent_primary
                } else {
                    colors.border_light
                },
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Shadow {
                color: if is_dark {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.1)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.05)
                },
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        }
    }
}

pub fn status_badge_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Default::default(),
        ..Default::default()
    }
}

// ============== Button Styles ==============

pub fn primary_button(colors: ColorScheme) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.accent_primary)),
            text_color: Color::from_rgb(0.1, 0.1, 0.1),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
            ..Default::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(colors.accent_hover)),
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.accent_dark)),
                ..base
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(colors.bg_tertiary)),
                text_color: colors.text_disabled,
                ..base
            },
            _ => base,
        }
    }
}

pub fn secondary_button(colors: ColorScheme) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.surface)),
            text_color: colors.text_primary,
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
            ..Default::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(colors.surface_hover)),
                border: Border {
                    color: colors.accent_primary,
                    ..base.border
                },
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.surface_active)),
                ..base
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(colors.bg_tertiary)),
                text_color: colors.text_disabled,
                ..base
            },
            _ => base,
        }
    }
}

pub fn danger_button(colors: ColorScheme) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(colors.surface)),
            text_color: colors.text_secondary,
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
            ..Default::default()
        };
        let error_color = Color::from_rgba(0.85, 0.25, 0.25, 1.0);
        let error_pressed = Color::from_rgba(0.75, 0.20, 0.20, 1.0);
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(error_color)),
                text_color: Color::WHITE,
                border: Border {
                    color: error_color,
                    ..base.border
                },
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(error_pressed)),
                text_color: Color::WHITE,
                border: Border {
                    color: error_pressed,
                    ..base.border
                },
                ..base
            },
            button::Status::Disabled => button::Style {
                background: Some(Background::Color(colors.bg_tertiary)),
                text_color: colors.text_disabled,
                ..base
            },
            _ => base,
        }
    }
}

pub fn icon_button(colors: ColorScheme) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors.text_secondary,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            shadow: Default::default(),
            ..Default::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Background::Color(colors.surface_hover)),
                text_color: colors.accent_primary,
                ..base
            },
            button::Status::Pressed => button::Style {
                background: Some(Background::Color(colors.surface_active)),
                text_color: colors.accent_dark,
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: colors.text_disabled,
                ..base
            },
            _ => base,
        }
    }
}

pub fn filter_button(
    colors: ColorScheme,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        if is_active {
            let base = button::Style {
                background: Some(Background::Color(colors.accent_primary)),
                text_color: Color::from_rgb(0.1, 0.1, 0.1),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(colors.accent_hover)),
                    ..base
                },
                _ => base,
            }
        } else {
            let base = button::Style {
                background: Some(Background::Color(colors.bg_tertiary)),
                text_color: colors.text_secondary,
                border: Border {
                    color: colors.border_light,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                ..Default::default()
            };
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(colors.bg_hover)),
                    text_color: colors.text_primary,
                    border: Border {
                        color: colors.border,
                        ..base.border
                    },
                    ..base
                },
                _ => base,
            }
        }
    }
}

pub fn transparent_button(
    colors: ColorScheme,
    is_selected: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: colors.text_primary,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: Default::default(),
            ..Default::default()
        };
        match status {
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(Background::Color(if is_selected {
                    Color::TRANSPARENT
                } else {
                    Color::from_rgba(0.5, 0.5, 0.5, 0.05)
                })),
                ..base
            },
            _ => base,
        }
    }
}

// ============== Input Styles ==============

pub fn text_input_style(
    colors: ColorScheme,
) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| match status {
        text_input::Status::Active => text_input::Style {
            background: Background::Color(colors.bg_secondary),
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: colors.text_secondary,
            placeholder: colors.text_disabled,
            value: colors.text_primary,
            selection: Color::from_rgba(0.95, 0.75, 0.25, 0.3),
        },
        text_input::Status::Hovered => text_input::Style {
            background: Background::Color(colors.bg_secondary),
            border: Border {
                color: colors.border_focus,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: colors.text_secondary,
            placeholder: colors.text_disabled,
            value: colors.text_primary,
            selection: Color::from_rgba(0.95, 0.75, 0.25, 0.3),
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: Background::Color(colors.bg_secondary),
            border: Border {
                color: colors.border_focus,
                width: 2.0,
                radius: 8.0.into(),
            },
            icon: colors.accent_primary,
            placeholder: colors.text_disabled,
            value: colors.text_primary,
            selection: Color::from_rgba(0.95, 0.75, 0.25, 0.3),
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(colors.bg_tertiary),
            border: Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: colors.text_disabled,
            placeholder: colors.text_disabled,
            value: colors.text_disabled,
            selection: Color::from_rgba(0.95, 0.75, 0.25, 0.3),
        },
    }
}

// ============== Progress Bar Styles ==============

pub fn progress_style(colors: ColorScheme) -> impl Fn(&Theme) -> progress_bar::Style {
    move |_theme| progress_bar::Style {
        background: Background::Color(colors.bg_tertiary),
        bar: Background::Color(colors.accent_primary),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
    }
}

pub fn progress_paused_style(colors: ColorScheme) -> impl Fn(&Theme) -> progress_bar::Style {
    move |_theme| progress_bar::Style {
        background: Background::Color(colors.bg_tertiary),
        bar: Background::Color(colors.warning),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
    }
}

pub fn progress_complete_style(colors: ColorScheme) -> impl Fn(&Theme) -> progress_bar::Style {
    move |_theme| progress_bar::Style {
        background: Background::Color(colors.bg_tertiary),
        bar: Background::Color(colors.success),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
    }
}

pub fn progress_error_style(colors: ColorScheme) -> impl Fn(&Theme) -> progress_bar::Style {
    move |_theme| progress_bar::Style {
        background: Background::Color(colors.bg_tertiary),
        bar: Background::Color(colors.error),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
    }
}

// ============== Scrollable Style ==============

pub fn scrollable_style(
    _colors: ColorScheme,
) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(Color::TRANSPARENT),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 3.0.into(),
            },
        },
    };
    let auto_scroll = scrollable::AutoScroll {
        background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.9)),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.8),
            width: 1.0,
            radius: u32::MAX.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
            offset: Vector::ZERO,
            blur_radius: 2.0,
        },
        icon: Color::from_rgba(1.0, 1.0, 1.0, 0.8),
    };
    move |_theme, _status| scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll,
    }
}

// ============== Checkbox / Toggle Style ==============

pub fn toggle_style(colors: ColorScheme) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
    move |_theme, status| {
        let (is_checked, is_hovered) = match status {
            checkbox::Status::Active { is_checked } => (is_checked, false),
            checkbox::Status::Hovered { is_checked } => (is_checked, true),
            checkbox::Status::Disabled { is_checked } => (is_checked, false),
        };

        let bg = if is_hovered {
            if is_checked {
                Background::Color(colors.accent_hover)
            } else {
                Background::Color(colors.bg_hover)
            }
        } else if is_checked {
            Background::Color(colors.accent_primary)
        } else {
            Background::Color(colors.bg_tertiary)
        };

        checkbox::Style {
            background: bg,
            icon_color: if is_checked {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                Color::WHITE
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            text_color: Some(colors.text_primary),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_effective_system_dark() {
        assert_eq!(ThemeMode::System.effective(true), ThemeMode::Dark);
    }

    #[test]
    fn theme_mode_effective_system_light() {
        assert_eq!(ThemeMode::System.effective(false), ThemeMode::Light);
    }

    #[test]
    fn theme_mode_effective_dark_passthrough() {
        assert_eq!(ThemeMode::Dark.effective(false), ThemeMode::Dark);
        assert_eq!(ThemeMode::Dark.effective(true), ThemeMode::Dark);
    }

    #[test]
    fn theme_mode_effective_light_passthrough() {
        assert_eq!(ThemeMode::Light.effective(false), ThemeMode::Light);
        assert_eq!(ThemeMode::Light.effective(true), ThemeMode::Light);
    }

    #[test]
    fn theme_mode_labels() {
        assert_eq!(ThemeMode::Dark.label(), "Dark");
        assert_eq!(ThemeMode::Light.label(), "Light");
        assert_eq!(ThemeMode::System.label(), "System");
    }

    #[test]
    fn theme_mode_default_is_dark() {
        assert_eq!(ThemeMode::default(), ThemeMode::Dark);
    }

    #[test]
    fn get_colors_dark_mode() {
        let colors = get_colors(ThemeMode::Dark);
        assert!(colors.bg_primary.r < 0.2, "dark bg should have low R");
    }

    #[test]
    fn get_colors_light_mode() {
        let colors = get_colors(ThemeMode::Light);
        assert!(colors.bg_primary.r > 0.9, "light bg should have high R");
    }

    #[test]
    fn get_colors_system_defaults_dark() {
        let dark = get_colors(ThemeMode::Dark);
        let system = get_colors(ThemeMode::System);
        assert_eq!(dark.bg_primary.r, system.bg_primary.r);
    }

    #[test]
    fn bolt_theme_produces_theme() {
        let theme = bolt_theme(ThemeMode::Dark);
        assert_eq!(
            theme.palette().background,
            get_colors(ThemeMode::Dark).bg_primary
        );
    }

    #[test]
    fn color_scheme_dark_accent() {
        let dark = ColorScheme::dark();
        assert!(dark.accent_primary.r > 0.9);
        assert!(dark.accent_primary.g > 0.7);
    }

    #[test]
    fn color_scheme_light_accent() {
        let light = ColorScheme::light();
        assert!(light.accent_primary.r > 0.8);
    }

    #[test]
    fn theme_mode_serde_roundtrip() {
        let mode = ThemeMode::Light;
        let json = serde_json::to_string(&mode).unwrap();
        let restored: ThemeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, mode);
    }
}
