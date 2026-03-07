#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bolt::BoltApp;
use iced::{Application, Font, Pixels, Settings};

const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

fn main() -> iced::Result {
    let fira_sans_font = Font::with_name("Fira Sans");

    BoltApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1000.0, 650.0),
            min_size: Some(iced::Size::new(750.0, 450.0)),
            exit_on_close_request: false,
            ..Default::default()
        },
        fonts: vec![iced_aw::BOOTSTRAP_FONT_BYTES.into(), JETBRAINS_MONO.into()],
        default_font: fira_sans_font,
        default_text_size: Pixels(15.0),
        ..Default::default()
    })
}
