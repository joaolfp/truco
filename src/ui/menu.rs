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
use super::UpdateState;
use super::widgets::BOLD;
use super::widgets::GOLD;
use super::widgets::MUTED;
use super::widgets::card_face;
use crate::game::Card;
use crate::game::MINEIRO_MANILHAS;
use crate::game::Rank;
use crate::game::Suit;
use crate::game::Variant;
use crate::i18n::Language;
use crate::update::CURRENT_VERSION;

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

/// Version label plus the "check for updates" button and its progress.
fn update_section(language: Language, update: &UpdateState) -> Element<'static, Message> {
	let small_button =
		|label: String,
		 message: Option<Message>,
		 style: fn(&iced::Theme, button::Status) -> button::Style| {
			button(text(label).size(15).font(BOLD))
				.padding([6, 14])
				.style(style)
				.on_press_maybe(message)
		};

	let (status, action) = match update {
		UpdateState::Idle => (
			None,
			small_button(
				language.check_for_updates().into(),
				Some(Message::CheckForUpdate),
				button::secondary,
			),
		),
		UpdateState::Checking => (
			None,
			small_button(
				language.checking_for_updates().into(),
				None,
				button::secondary,
			),
		),
		UpdateState::UpToDate => (
			Some(language.up_to_date().to_owned()),
			small_button(
				language.check_for_updates().into(),
				Some(Message::CheckForUpdate),
				button::secondary,
			),
		),
		UpdateState::Available(release) => (
			Some(language.update_available().to_owned()),
			small_button(
				language.update_to(release.version()),
				Some(Message::InstallUpdate),
				button::success,
			),
		),
		UpdateState::Installing(release) => (
			None,
			small_button(
				language.installing_update(release.version()),
				None,
				button::success,
			),
		),
		UpdateState::Handed => (
			Some(language.update_handed().to_owned()),
			small_button(
				language.check_for_updates().into(),
				Some(Message::CheckForUpdate),
				button::secondary,
			),
		),
		UpdateState::Failed(error) => (
			Some(language.update_failed(error)),
			small_button(
				language.try_again().into(),
				Some(Message::CheckForUpdate),
				button::secondary,
			),
		),
	};

	let mut section = column![
		row![
			text(format!("v{CURRENT_VERSION}")).size(15).color(MUTED),
			action
		]
		.spacing(12)
		.align_y(Center),
	]
	.spacing(6)
	.align_x(Center);
	if let Some(status) = status {
		section = section.push(
			text(status)
				.size(14)
				.color(MUTED)
				.width(420)
				.align_x(Center),
		);
	}
	section.into()
}

pub fn view(
	language: Language,
	variant: Variant,
	has_game: bool,
	update: &UpdateState,
) -> Element<'static, Message> {
	let strings = language.strings();
	let showcase = match variant {
		// With a 3 as vira the 4s are manilhas, so the zap and copas shine in gold.
		Variant::Paulista => row![
			card_face(Card::new(Rank::Four, Suit::Clubs), true, 70.0),
			card_face(Card::new(Rank::Four, Suit::Hearts), true, 70.0),
			card_face(Card::new(Rank::Three, Suit::Spades), false, 70.0),
			card_face(Card::new(Rank::Two, Suit::Diamonds), false, 70.0),
		],
		// Mineiro's fixed manilhas, strongest first.
		Variant::Mineiro => MINEIRO_MANILHAS
			.iter()
			.rev()
			.fold(row![], |row, &card| row.push(card_face(card, true, 70.0))),
	}
	.spacing(12);

	let mode_buttons = Variant::ALL.iter().fold(row![].spacing(8), |row, &option| {
		let style = if option == variant {
			button::primary
		} else {
			button::secondary
		};
		row.push(
			button(
				text(language.variant_name(option).trim_start_matches("Truco "))
					.size(17)
					.font(BOLD)
					.width(Fill)
					.align_x(Center),
			)
			.width(136)
			.padding(10)
			.style(style)
			.on_press(Message::SetVariant(option)),
		)
	});
	let mode_picker = column![
		text(strings.game_mode).color(MUTED),
		mode_buttons,
		text(language.variant_summary(variant))
			.size(14)
			.color(MUTED),
	]
	.spacing(8)
	.align_x(Center);

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
		mode_picker,
		buttons,
		language_picker,
		update_section(language, update),
	]
	.spacing(20)
	.align_x(Center);

	container(content).center(Fill).padding(24).into()
}
