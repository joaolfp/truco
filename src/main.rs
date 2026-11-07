//! Truco Paulista: play against a bot and learn the rules step by step.

mod bot;
mod game;
mod i18n;
mod opponents;
mod ui;

use ui::App;

fn main() -> iced::Result {
	iced::application(App::boot, App::update, App::view)
		.title(App::title)
		.theme(App::theme)
		.subscription(App::subscription)
		.window_size((1280.0, 860.0))
		.centered()
		.run()
}
