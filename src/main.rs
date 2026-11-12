//! Truco Paulista: play against a bot and learn the rules step by step.

// Release builds on Windows open only the game window, without a console.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bot;
mod game;
mod i18n;
mod opponents;
mod sound;
mod ui;
mod update;

use iced::window;
use ui::App;

fn main() -> iced::Result {
	iced::application(App::boot, App::update, App::view)
		.title(App::title)
		.theme(App::theme)
		.subscription(App::subscription)
		.window(window::Settings {
			size: (1280.0, 860.0).into(),
			position: window::Position::Centered,
			icon: window_icon(),
			..window::Settings::default()
		})
		.run()
}

/// The app icon shown in the title bar and taskbar (Windows and Linux).
fn window_icon() -> Option<window::Icon> {
	let decoder = png::Decoder::new(std::io::Cursor::new(include_bytes!(
		"../assets/icon-64.png"
	)));
	let mut reader = decoder.read_info().ok()?;
	let mut rgba = vec![0; reader.output_buffer_size()?];
	let info = reader.next_frame(&mut rgba).ok()?;
	if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
		return None;
	}
	rgba.truncate(info.buffer_size());
	window::icon::from_rgba(rgba, info.width, info.height).ok()
}
