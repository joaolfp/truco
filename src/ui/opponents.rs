//! Screen where the player picks who to challenge before a match.

use iced::Center;
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;

use super::Message;
use super::widgets::BOLD;
use super::widgets::GOLD;
use super::widgets::MUTED;
use super::widgets::avatar;
use super::widgets::panel;
use crate::game::Variant;
use crate::i18n::Language;
use crate::opponents::Difficulty;
use crate::opponents::OPPONENTS;
use crate::opponents::Opponent;

fn stars(level: usize) -> String {
	(1..=Difficulty::ALL.len())
		.map(|i| if i <= level { '★' } else { '☆' })
		.collect()
}

fn opponent_card(opponent: Opponent, language: Language) -> Element<'static, Message> {
	let level = opponent.difficulty.level();
	let color = opponent.color;

	let content = column![
		avatar(&opponent, 84.0),
		text(opponent.name).size(26).font(BOLD),
		container(
			text(language.difficulty_name(opponent.difficulty))
				.size(15)
				.font(BOLD)
				.color(Color::WHITE),
		)
		.padding([3, 12])
		.style(move |_| container::Style {
			background: Some(color.into()),
			border: iced::Border::default().rounded(12),
			..container::Style::default()
		}),
		text(stars(level)).size(22).color(GOLD),
		text(language.opponent_bio(opponent.difficulty))
			.size(15)
			.color(MUTED)
			.align_x(Center),
		space::vertical(),
		button(
			text(language.strings().challenge)
				.size(18)
				.font(BOLD)
				.width(Fill)
				.align_x(Center),
		)
		.width(Fill)
		.padding(10)
		.style(button::primary)
		.on_press(Message::Challenge(opponent.difficulty)),
	]
	.spacing(12)
	.align_x(Center);

	container(content)
		.padding(20)
		.width(260)
		.height(440)
		.style(panel)
		.into()
}

pub fn view(language: Language, variant: Variant) -> Element<'static, Message> {
	let strings = language.strings();

	let header = row![
		button(text(strings.back))
			.on_press(Message::OpenMenu)
			.style(button::secondary),
		text(strings.choose_opponent)
			.size(30)
			.font(BOLD)
			.color(GOLD),
		space::horizontal(),
		text(format!(
			"{}: {}",
			strings.game_mode,
			language.variant_name(variant)
		))
		.color(MUTED),
	]
	.spacing(16)
	.align_y(Center);

	let cards = OPPONENTS
		.into_iter()
		.fold(row![].spacing(20), |row, opponent| {
			row.push(opponent_card(opponent, language))
		});

	column![header, container(cards).center(Fill)]
		.spacing(16)
		.padding(20)
		.into()
}
