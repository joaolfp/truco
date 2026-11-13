//! Step-by-step tutorial with illustrations and small quizzes.

use iced::Center;
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::progress_bar;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;

use super::Message;
use super::widgets::BOLD;
use super::widgets::BOT_COLOR;
use super::widgets::GOLD;
use super::widgets::HUMAN_COLOR;
use super::widgets::MUTED;
use super::widgets::badge;
use super::widgets::card_button;
use super::widgets::card_face;
use super::widgets::panel;
use crate::game::Card;
use crate::game::MINEIRO_MANILHAS;
use crate::game::Rank;
use crate::game::Stake;
use crate::game::Suit;
use crate::game::Variant;
use crate::i18n::Language;

pub const STEP_COUNT: usize = 13;

const fn card(rank: Rank, suit: Suit) -> Card {
	Card::new(rank, suit)
}

/// A "click the winning card" question.
struct Quiz {
	vira: Card,
	options: [Card; 2],
	answer: usize,
}

fn quiz_for(step: usize) -> Option<Quiz> {
	match step {
		// Vira K → manilha is A, so the humble ace beats the 3.
		5 => Some(Quiz {
			vira: card(Rank::King, Suit::Spades),
			options: [
				card(Rank::Three, Suit::Clubs),
				card(Rank::Ace, Suit::Diamonds),
			],
			answer: 1,
		}),
		// Vira 3 → manilha is 4; hearts beats spades.
		9 => Some(Quiz {
			vira: card(Rank::Three, Suit::Diamonds),
			options: [
				card(Rank::Four, Suit::Hearts),
				card(Rank::Four, Suit::Spades),
			],
			answer: 0,
		}),
		_ => None,
	}
}

fn caption<'a>(label: impl text::IntoFragment<'a>) -> Element<'a, Message> {
	text(label).size(14).color(MUTED).into()
}

fn labeled<'a>(
	card_view: Element<'a, Message>,
	label: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
	column![card_view, caption(label)]
		.spacing(6)
		.align_x(Center)
		.into()
}

fn illustration(
	step: usize,
	language: Language,
	quiz_answer: Option<usize>,
) -> Element<'static, Message> {
	let strings = language.strings();
	let small = 58.0;
	let medium = 80.0;

	match step {
		0 => row![
			card_face(card(Rank::Three, Suit::Spades), false, medium),
			card_face(card(Rank::Four, Suit::Clubs), true, medium),
			card_face(card(Rank::Ace, Suit::Hearts), false, medium),
		]
		.spacing(16)
		.into(),
		1 => {
			let suits = Suit::ALL;
			Rank::ALL
				.iter()
				.enumerate()
				.fold(row![].spacing(8), |row, (i, &rank)| {
					row.push(card_face(card(rank, suits[i % 4]), false, small))
				})
				.into()
		}
		2 => {
			let cards = Rank::ALL
				.iter()
				.enumerate()
				.fold(row![].spacing(8), |row, (i, &rank)| {
					row.push(card_face(card(rank, Suit::ALL[i % 4]), false, small))
				});
			column![
				cards,
				row![
					caption(format!("← {}", strings.weakest)),
					space::horizontal(),
					caption(format!("{} →", strings.strongest)),
				]
				.width(Fill),
			]
			.spacing(8)
			.width(10.0 * (small + 8.0))
			.into()
		}
		3 => {
			let manilhas = Suit::ALL.iter().fold(row![].spacing(8), |row, &suit| {
				row.push(card_face(card(Rank::Queen, suit), true, small))
			});
			row![
				labeled(
					card_face(card(Rank::Seven, Suit::Hearts), false, medium),
					strings.vira
				),
				text("→").size(40).color(GOLD),
				labeled(manilhas.into(), strings.manilha),
			]
			.spacing(20)
			.align_y(Center)
			.into()
		}
		4 => Suit::ALL
			.iter()
			.fold(row![].spacing(14).align_y(Center), |row, &suit| {
				let label = strings.manilha_names[suit.manilha_order() as usize];
				let row = if suit == Suit::Diamonds {
					row
				} else {
					row.push(text("<").size(28).color(GOLD))
				};

				row.push(labeled(
					card_face(card(Rank::Queen, suit), true, medium),
					label,
				))
			})
			.into(),
		5 | 9 => {
			let Some(quiz) = quiz_for(step) else {
				return space().into();
			};

			let options =
				quiz
					.options
					.iter()
					.enumerate()
					.fold(row![].spacing(20), |row, (index, &option)| {
						let reveal = quiz_answer == Some(quiz.answer) && index == quiz.answer;
						row.push(card_button(
							option,
							reveal.then_some(GOLD),
							medium,
							Some(Message::QuizAnswer(index)),
						))
					});
			let feedback: Element<'_, Message> = match quiz_answer {
				Some(answer) if answer == quiz.answer => text(strings.quiz_correct)
					.size(20)
					.font(BOLD)
					.color(HUMAN_COLOR)
					.into(),
				Some(_) => text(strings.quiz_wrong)
					.size(18)
					.font(BOLD)
					.color(BOT_COLOR)
					.into(),
				None => space().height(24).into(),
			};

			row![
				labeled(card_face(quiz.vira, false, medium), strings.vira),
				column![text(strings.quiz_prompt).font(BOLD), options, feedback]
					.spacing(12)
					.align_x(Center),
			]
			.spacing(40)
			.align_y(Center)
			.into()
		}
		6 => row![
			labeled(
				card_face(card(Rank::Two, Suit::Spades), false, medium),
				strings.you
			),
			text("vs").size(24).color(MUTED),
			labeled(
				card_face(card(Rank::Jack, Suit::Hearts), false, medium),
				strings.bot
			),
			text("→").size(32).color(GOLD),
			badge(strings.you, HUMAN_COLOR),
		]
		.spacing(16)
		.align_y(Center)
		.into(),
		7 => row![
			card_face(card(Rank::Three, Suit::Diamonds), false, medium),
			text("=").size(40).color(GOLD),
			card_face(card(Rank::Three, Suit::Clubs), false, medium),
		]
		.spacing(16)
		.align_y(Center)
		.into(),
		8 => stake_ladder(language, Variant::Paulista),
		10 => row![
			badge(format!("{} 11", strings.you), HUMAN_COLOR),
			text("x").size(24).color(MUTED),
			badge(format!("7 {}", strings.bot), BOT_COLOR),
		]
		.spacing(12)
		.align_y(Center)
		.into(),
		11 => {
			let manilhas = MINEIRO_MANILHAS.iter().rev().enumerate().fold(
				row![].spacing(10).align_y(Center),
				|row, (i, &manilha)| {
					let row = if i == 0 {
						row
					} else {
						row.push(text(">").size(24).color(GOLD))
					};

					row.push(card_face(manilha, true, small))
				},
			);
			column![manilhas, stake_ladder(language, Variant::Mineiro)]
				.spacing(18)
				.align_x(Center)
				.into()
		}
		_ => row![
			card_face(card(Rank::Four, Suit::Clubs), true, medium),
			card_face(card(Rank::Seven, Suit::Hearts), false, medium),
			card_face(card(Rank::Ace, Suit::Spades), false, medium),
		]
		.spacing(16)
		.into(),
	}
}

/// The raise ladder with the points of each call for a variant.
fn stake_ladder(language: Language, variant: Variant) -> Element<'static, Message> {
	[
		Stake::Base,
		Stake::Truco,
		Stake::Six,
		Stake::Nine,
		Stake::Twelve,
	]
	.into_iter()
	.enumerate()
	.fold(row![].spacing(10).align_y(Center), |row, (i, stake)| {
		let row = if i == 0 {
			row
		} else {
			row.push(text("→").size(24).color(GOLD))
		};

		let name = if stake == Stake::Base {
			String::from("—")
		} else {
			language.raise_button(stake)
		};

		row.push(
			column![
				badge(
					variant.stake_value(stake).to_string(),
					Color::from_rgb(0.75, 0.45, 0.05)
				),
				caption(name)
			]
			.spacing(4)
			.align_x(Center),
		)
	})
	.into()
}

pub fn view(
	language: Language,
	step: usize,
	quiz_answer: Option<usize>,
) -> Element<'static, Message> {
	let strings = language.strings();
	let current = &strings.tutorial_steps[step];
	let is_last = step + 1 == STEP_COUNT;
	let quiz_blocks_next = quiz_for(step).is_some_and(|quiz| quiz_answer != Some(quiz.answer));

	let header = row![
		button(strings.menu)
			.on_press(Message::OpenMenu)
			.style(button::secondary),
		text(strings.tutorial).size(28).font(BOLD).color(GOLD),
		space::horizontal(),
		text(language.tutorial_progress(step + 1, STEP_COUNT)).color(MUTED),
		pick_list(Language::ALL, Some(language), Message::SetLanguage),
	]
	.spacing(16)
	.align_y(Center);

	let progress = progress_bar(0.0..=STEP_COUNT as f32, (step + 1) as f32).girth(8);

	let body = container(
		column![
			text(current.title).size(32).font(BOLD),
			text(current.body).size(19),
			container(illustration(step, language, quiz_answer))
				.center_x(Fill)
				.padding(12),
		]
		.spacing(20),
	)
	.padding(28)
	.max_width(900)
	.style(panel);

	let previous = button(text(strings.previous).size(18))
		.padding([10, 22])
		.style(button::secondary)
		.on_press_maybe((step > 0).then_some(Message::TutorialPrevious));
	let next = if is_last {
		button(text(strings.start_playing).size(18).font(BOLD))
			.padding([10, 22])
			.style(button::success)
			.on_press(Message::NewGame)
	} else {
		button(text(strings.next).size(18).font(BOLD))
			.padding([10, 22])
			.style(button::primary)
			.on_press_maybe((!quiz_blocks_next).then_some(Message::TutorialNext))
	};

	let navigation = container(row![previous, space::horizontal(), next])
		.width(Fill)
		.max_width(900);

	column![
		header,
		progress,
		scrollable(container(body).center_x(Fill)).height(Fill),
		container(navigation).center_x(Fill),
	]
	.spacing(16)
	.padding(20)
	.into()
}
