#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bolt::BoltApp;
use iced::Font;

const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

fn main() -> iced::Result {
    iced::daemon(BoltApp::boot, BoltApp::update, BoltApp::view)
        .title(BoltApp::title)
        .subscription(BoltApp::subscription)
        .theme(BoltApp::theme)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .font(JETBRAINS_MONO)
        .default_font(Font::with_name("Fira Sans"))
        .run()
}
