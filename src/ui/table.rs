//! The game screen: scores, the table, the player's hand and actions.

use iced::Center;
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::Column;
use iced::widget::button;
use iced::widget::canvas;
use iced::widget::column;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::stack;
use iced::widget::text;
use iced::widget::toggler;

use super::Message;
use super::scene;
use super::widgets::BOLD;
use super::widgets::BOT_COLOR;
use super::widgets::GOLD;
use super::widgets::HINT;
use super::widgets::HUMAN_COLOR;
use super::widgets::MUTED;
use super::widgets::avatar;
use super::widgets::badge;
use super::widgets::card_back;
use super::widgets::card_button;
use super::widgets::card_face;
use super::widgets::chalkboard;
use super::widgets::hint_panel;
use super::widgets::panel;
use super::widgets::round_marker;
use crate::bot;
use crate::game::Action;
use crate::game::Event;
use crate::game::Game;
use crate::game::MINEIRO_MANILHAS;
use crate::game::Phase;
use crate::game::Player;
use crate::game::TrickResult;
use crate::i18n::Language;
use crate::opponents::Opponent;

const HAND_CARD: f32 = 104.0;

pub fn view<'a>(
	game: &'a Game,
	opponent: Opponent,
	language: Language,
	hints: bool,
	sound: bool,
) -> Element<'a, Message> {
	let suggestion = (hints && game.actor() == Some(Player::Human))
		.then(|| bot::decide(game, Player::Human, None))
		.flatten();

	// The round table is drawn on a canvas; the bot's cards, prompts and the
	// player's hand float over it.
	let overlay = column![
		opponent_area(game, opponent, language),
		space::vertical(),
		status_line(game, opponent, language),
		actions(game, language),
		player_hand(game, hints, suggestion.map(|s| s.action)),
	]
	.spacing(12)
	.align_x(Center)
	.width(Fill)
	.height(Fill);
	let table = stack![
		canvas(scene::Table { game, hints })
			.width(Fill)
			.height(Fill),
		overlay,
	];
	let main = column![top_bar(game, opponent, language, hints, sound), table]
		.spacing(12)
		.width(Fill);

	let mut side = column![info_panel(game, opponent, language)]
		.spacing(16)
		.width(300);
	if let Some(suggestion) = suggestion {
		side = side.push(
			container(
				column![
					text(language.strings().hint)
						.font(BOLD)
						.size(18)
						.color(HINT),
					text(language.suggestion(suggestion.action, game))
						.font(BOLD)
						.size(17),
					text(language.reason(suggestion.reason)).size(15),
				]
				.spacing(6),
			)
			.padding(14)
			.width(Fill)
			.style(hint_panel),
		);
	}
	side = side.push(event_log(game, opponent, language));

	row![main, side].spacing(16).padding(16).into()
}

fn top_bar<'a>(
	game: &'a Game,
	opponent: Opponent,
	language: Language,
	hints: bool,
	sound: bool,
) -> Element<'a, Message> {
	let strings = language.strings();
	let score = container(
		row![
			text(strings.you).size(18).color(HUMAN_COLOR).font(BOLD),
			text(game.score(Player::Human)).size(30).font(BOLD),
			text("x").size(18).color(MUTED),
			text(game.score(Player::Bot)).size(30).font(BOLD),
			text(opponent.name).size(18).color(BOT_COLOR).font(BOLD),
		]
		.spacing(12)
		.align_y(Center),
	)
	.padding([4, 16])
	.style(chalkboard);

	row![
		text("TRUCO").size(28).font(BOLD).color(GOLD),
		space::horizontal(),
		score,
		badge(
			language.hand_worth(game.stake_value(game.stake)),
			Color::from_rgb(0.75, 0.45, 0.05)
		),
		space::horizontal(),
		toggler(hints)
			.label(strings.hints)
			.on_toggle(Message::ToggleHints),
		toggler(sound)
			.label(strings.sound)
			.on_toggle(Message::ToggleSound),
		pick_list(Language::ALL, Some(language), Message::SetLanguage),
		button(strings.menu)
			.on_press(Message::OpenMenu)
			.style(button::secondary),
	]
	.spacing(14)
	.align_y(Center)
	.into()
}

fn opponent_area<'a>(
	game: &'a Game,
	opponent: Opponent,
	language: Language,
) -> Element<'a, Message> {
	let cards = game
		.hand(Player::Bot)
		.iter()
		.fold(row![].spacing(8), |row, _| row.push(card_back(60.0)));
	let who = row![
		avatar(&opponent, 44.0),
		column![
			text(opponent.name).font(BOLD).size(18),
			text(language.difficulty_name(opponent.difficulty))
				.size(13)
				.color(MUTED),
		]
		.spacing(2),
	]
	.spacing(10)
	.align_y(Center);
	let who = container(who).padding([6, 12]).style(panel);
	row![who, cards]
		.spacing(16)
		.align_y(Center)
		.height(100)
		.into()
}

fn info_panel<'a>(game: &'a Game, opponent: Opponent, language: Language) -> Element<'a, Message> {
	let strings = language.strings();

	// Paulista shows the vira and the manilha rank it defines; Mineiro shows
	// its four fixed manilhas, strongest first.
	let manilhas: Element<'_, Message> = match game.vira {
		Some(vira) => row![
			column![
				text(strings.vira).font(BOLD).color(MUTED),
				card_face(vira, false, 52.0),
			]
			.spacing(6)
			.align_x(Center),
			text(format!("{}: {}", strings.manilha, vira.rank.next().label()))
				.font(BOLD)
				.size(18)
				.color(GOLD),
		]
		.spacing(18)
		.align_y(Center)
		.into(),
		None => column![
			text(strings.manilhas).font(BOLD).color(MUTED),
			MINEIRO_MANILHAS
				.iter()
				.rev()
				.fold(row![].spacing(8), |row, &card| row
					.push(card_face(card, true, 46.0))),
		]
		.spacing(6)
		.into(),
	};

	let markers = (0..3).fold(row![].spacing(8), |row, index| {
		let fill = game.tricks.get(index).map(|result| match result {
			TrickResult::Won(Player::Human) => HUMAN_COLOR,
			TrickResult::Won(Player::Bot) => BOT_COLOR,
			TrickResult::Tied => Color::from_rgb(0.6, 0.6, 0.6),
		});
		row.push(round_marker(fill))
	});
	let rounds = row![
		text(strings.rounds).font(BOLD).color(MUTED),
		markers,
		column![
			text(format!("● {}", strings.you))
				.size(13)
				.color(HUMAN_COLOR),
			text(format!("● {}", opponent.name))
				.size(13)
				.color(BOT_COLOR),
		],
	]
	.spacing(12)
	.align_y(Center);

	container(
		column![
			text(language.variant_name(game.variant))
				.font(BOLD)
				.size(16),
			manilhas,
			rounds,
		]
		.spacing(12),
	)
	.padding(14)
	.width(Fill)
	.style(panel)
	.into()
}

fn status_line<'a>(game: &'a Game, opponent: Opponent, language: Language) -> Element<'a, Message> {
	let strings = language.strings();
	let (message, color) = match game.phase {
		Phase::MatchOver { winner } => match winner {
			Player::Human => (strings.you_won_match.to_string(), GOLD),
			Player::Bot => (language.opponent_won_match(opponent.name), BOT_COLOR),
		},
		Phase::HandOver => {
			let summary = game
				.events
				.iter()
				.rev()
				.find(|event| matches!(event, Event::HandWon { .. } | Event::HandDrawn))
				.map(|event| language.event(event, game.variant, opponent.name))
				.unwrap_or_default();
			(summary, GOLD)
		}
		Phase::ElevenDecision {
			player: Player::Human,
		} => (
			language.special_hand_prompt(game.variant, opponent.name),
			GOLD,
		),
		Phase::AwaitingResponse {
			responder: Player::Human,
			proposed,
		} => (
			language.bot_raised(proposed, game.variant, opponent.name),
			GOLD,
		),
		Phase::Playing if game.turn == Player::Human => (strings.your_turn.to_string(), Color::WHITE),
		_ => (language.thinking(opponent.name), MUTED),
	};
	container(text(message).size(19).font(BOLD).color(color))
		.padding([6, 16])
		.style(panel)
		.into()
}

fn action_button(
	label: String,
	message: Message,
	style: fn(&iced::Theme, button::Status) -> button::Style,
) -> Element<'static, Message> {
	button(text(label).size(18).font(BOLD))
		.padding([10, 22])
		.style(style)
		.on_press(message)
		.into()
}

fn actions<'a>(game: &'a Game, language: Language) -> Element<'a, Message> {
	let strings = language.strings();
	let raise = game.raise_target(Player::Human).map(|stake| {
		action_button(
			language.raise_button(stake),
			Message::Act(Action::Raise),
			button::warning,
		)
	});

	let buttons: Vec<Element<'_, Message>> = match game.phase {
		Phase::MatchOver { .. } => vec![action_button(
			strings.new_match.into(),
			Message::NewGame,
			button::primary,
		)],
		Phase::HandOver => vec![action_button(
			strings.next_hand.into(),
			Message::NextHand,
			button::primary,
		)],
		Phase::ElevenDecision {
			player: Player::Human,
		} => vec![
			action_button(
				strings.play_eleven.into(),
				Message::Act(Action::Accept),
				button::success,
			),
			action_button(
				strings.run.into(),
				Message::Act(Action::Fold),
				button::danger,
			),
		],
		Phase::AwaitingResponse {
			responder: Player::Human,
			..
		} => {
			let mut buttons = vec![
				action_button(
					strings.accept.into(),
					Message::Act(Action::Accept),
					button::success,
				),
				action_button(
					strings.run.into(),
					Message::Act(Action::Fold),
					button::danger,
				),
			];
			buttons.extend(raise);
			buttons
		}
		Phase::Playing if game.turn == Player::Human => raise.into_iter().collect(),
		_ => Vec::new(),
	};

	row(buttons).spacing(12).height(48).align_y(Center).into()
}

/// Without hints no card is marked; with them manilhas get a gold border and
/// the suggested card a hint-colored one.
fn player_hand<'a>(game: &'a Game, hints: bool, suggested: Option<Action>) -> Element<'a, Message> {
	let can_play = game.phase == Phase::Playing && game.turn == Player::Human;
	let cards = game
		.hand(Player::Human)
		.iter()
		.enumerate()
		.map(|(index, &card)| {
			let action = Action::Play(index);
			let border = if suggested == Some(action) {
				Some(HINT)
			} else if hints && game.is_manilha(card) {
				Some(GOLD)
			} else {
				None
			};
			card_button(
				card,
				border,
				HAND_CARD,
				can_play.then_some(Message::Act(action)),
			)
		});
	container(row(cards).spacing(14))
		.height(HAND_CARD * 1.45 + 10.0)
		.center_x(Fill)
		.into()
}

fn event_log<'a>(game: &'a Game, opponent: Opponent, language: Language) -> Element<'a, Message> {
	let entries = game.events.iter().rev().take(40).map(|event| {
		let color = match event {
			Event::NewHand { .. } => GOLD,
			Event::HandWon { .. } | Event::MatchWon { .. } | Event::HandDrawn => Color::WHITE,
			_ => MUTED,
		};
		text(language.event(event, game.variant, opponent.name))
			.size(14)
			.color(color)
			.into()
	});
	container(
		column![
			text(language.strings().log).font(BOLD).size(18),
			scrollable(Column::with_children(entries).spacing(6)).height(Fill),
		]
		.spacing(10),
	)
	.padding(14)
	.width(Fill)
	.height(Fill)
	.style(panel)
	.into()
}
