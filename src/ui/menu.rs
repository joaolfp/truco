//! Main menu screen.

use iced::Center;
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::text;

use super::Message;
use super::widgets::BOLD;
use super::widgets::GOLD;
use super::widgets::MUTED;
use super::widgets::card_face;
use crate::game::Card;
use crate::game::Rank;
use crate::game::Suit;
use crate::i18n::Language;

fn menu_button(
	label: &str,
	message: Message,
	style: fn(&iced::Theme, button::Status) -> button::Style,
) -> Element<'_, Message> {
	button(text(label).size(20).font(BOLD).width(Fill).align_x(Center))
		.width(280)
		.padding(12)
		.style(style)
		.on_press(message)
		.into()
}

pub fn view(language: Language, has_game: bool) -> Element<'static, Message> {
	let strings = language.strings();
	// With a 3 as vira the 4s are manilhas, so the zap and copas shine in gold.
	let showcase = row![
		card_face(Card::new(Rank::Four, Suit::Clubs), true, 70.0),
		card_face(Card::new(Rank::Four, Suit::Hearts), true, 70.0),
		card_face(Card::new(Rank::Three, Suit::Spades), false, 70.0),
		card_face(Card::new(Rank::Two, Suit::Diamonds), false, 70.0),
	]
	.spacing(12);

	let mut buttons = column![].spacing(12).align_x(Center);
	if has_game {
		buttons = buttons.push(menu_button(
			strings.continue_match,
			Message::ContinueGame,
			button::primary,
		));
		buttons = buttons.push(menu_button(
			strings.new_match,
			Message::NewGame,
			button::secondary,
		));
	} else {
		buttons = buttons.push(menu_button(strings.play, Message::NewGame, button::primary));
	}
	buttons = buttons.push(menu_button(
		strings.tutorial,
		Message::OpenTutorial,
		button::success,
	));

	let language_picker = row![
		text(strings.language).color(MUTED),
		pick_list(Language::ALL, Some(language), Message::SetLanguage),
	]
	.spacing(10)
	.align_y(Center);

	let content = column![
		text("TRUCO").size(84).font(BOLD).color(GOLD),
		text("♠  ♥  ♦  ♣").size(28).color(Color::WHITE),
		text(strings.subtitle).size(20).color(MUTED),
		showcase,
		buttons,
		language_picker,
	]
	.spacing(24)
	.align_x(Center);

	container(content).center(Fill).padding(24).into()
}
