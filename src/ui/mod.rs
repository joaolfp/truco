//! The iced application: screens, messages and state updates.

#[cfg(feature = "screenshots")]
mod capture;
mod menu;
mod opponents;
mod scene;
mod table;
mod tutorial;
mod widgets;

use std::time::Duration;

use iced::Element;
use iced::Fill;
use iced::Subscription;
use iced::Task;
use iced::Theme;
use iced::widget::canvas;
use iced::widget::container;
use iced::widget::stack;

use crate::bot;
use crate::game::Action;
use crate::game::Game;
use crate::game::Player;
use crate::game::Variant;
use crate::i18n::Language;
use crate::opponents::Difficulty;
use crate::opponents::Opponent;

/// Delay between bot moves so the player can follow what happens.
const BOT_DELAY: Duration = Duration::from_millis(900);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
	Menu,
	Tutorial,
	Opponents,
	Game,
}

#[derive(Debug, Clone)]
pub enum Message {
	SetLanguage(Language),
	OpenMenu,
	OpenTutorial,
	ContinueGame,
	/// Go to the opponent selection before a new match.
	NewGame,
	Challenge(Difficulty),
	SetVariant(Variant),
	TutorialPrevious,
	TutorialNext,
	QuizAnswer(usize),
	ToggleHints(bool),
	Act(Action),
	NextHand,
	BotTick,
	#[cfg(feature = "screenshots")]
	Capture(capture::Step),
}

pub struct App {
	language: Language,
	screen: Screen,
	game: Option<Game>,
	/// Rules used when a new match starts.
	variant: Variant,
	/// Who the player is facing in the current match.
	opponent: Opponent,
	hints: bool,
	tutorial_step: usize,
	quiz_answer: Option<usize>,
	rng: fastrand::Rng,
	/// Set while the automated screenshot tour runs; pauses the bot.
	touring: bool,
}

impl Default for App {
	fn default() -> Self {
		Self {
			language: Language::default(),
			screen: Screen::Menu,
			game: None,
			variant: Variant::default(),
			opponent: Opponent::for_difficulty(Difficulty::default()),
			hints: false,
			tutorial_step: 0,
			quiz_answer: None,
			rng: fastrand::Rng::new(),
			touring: false,
		}
	}
}

impl App {
	pub fn boot() -> (Self, Task<Message>) {
		#[cfg(feature = "screenshots")]
		{
			let app = Self {
				touring: true,
				..Self::default()
			};
			(app, capture::start())
		}
		#[cfg(not(feature = "screenshots"))]
		{
			(Self::default(), Task::none())
		}
	}

	pub fn title(&self) -> String {
		match self.screen {
			Screen::Menu => "Truco".into(),
			Screen::Tutorial => format!("Truco — {}", self.language.strings().tutorial),
			Screen::Opponents => format!("Truco — {}", self.language.strings().choose_opponent),
			Screen::Game => {
				let Some(game) = &self.game else {
					return "Truco".into();
				};
				format!(
					"Truco — {} {} x {} {}",
					self.language.strings().you,
					game.score(Player::Human),
					game.score(Player::Bot),
					self.opponent.name
				)
			}
		}
	}

	pub fn theme(&self) -> Theme {
		Theme::Dark
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::SetLanguage(language) => self.language = language,
			Message::OpenMenu => self.screen = Screen::Menu,
			Message::OpenTutorial => {
				self.screen = Screen::Tutorial;
				self.tutorial_step = 0;
				self.quiz_answer = None;
			}
			Message::ContinueGame => {
				if self.game.is_some() {
					self.screen = Screen::Game;
				}
			}
			Message::SetVariant(variant) => self.variant = variant,
			Message::NewGame => self.screen = Screen::Opponents,
			Message::Challenge(difficulty) => {
				self.opponent = Opponent::for_difficulty(difficulty);
				self.game = Some(Game::new(self.variant));
				self.screen = Screen::Game;
			}
			Message::TutorialPrevious => {
				self.tutorial_step = self.tutorial_step.saturating_sub(1);
				self.quiz_answer = None;
			}
			Message::TutorialNext => {
				self.tutorial_step = (self.tutorial_step + 1).min(tutorial::STEP_COUNT - 1);
				self.quiz_answer = None;
			}
			Message::QuizAnswer(answer) => self.quiz_answer = Some(answer),
			Message::ToggleHints(enabled) => self.hints = enabled,
			Message::Act(action) => {
				if let Some(game) = &mut self.game {
					// Illegal clicks (e.g. a stale button) are simply ignored.
					let _ = game.apply(Player::Human, action);
				}
			}
			Message::NextHand => {
				if let Some(game) = &mut self.game {
					game.next_hand();
				}
			}
			Message::BotTick => {
				if let Some(game) = &mut self.game
					&& let Some(decision) =
						bot::decide_as(self.opponent.difficulty, game, Player::Bot, &mut self.rng)
				{
					let _ = game.apply(Player::Bot, decision.action);
				}
			}
			#[cfg(feature = "screenshots")]
			Message::Capture(step) => return capture::update(self, step),
		}
		Task::none()
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let bot_to_act = !self.touring
			&& self.screen == Screen::Game
			&& self.game.as_ref().and_then(Game::actor) == Some(Player::Bot);
		if bot_to_act {
			iced::time::every(BOT_DELAY).map(|_| Message::BotTick)
		} else {
			Subscription::none()
		}
	}

	pub fn view(&self) -> Element<'_, Message> {
		let content = match self.screen {
			Screen::Menu => menu::view(self.language, self.variant, self.game.is_some()),
			Screen::Tutorial => tutorial::view(self.language, self.tutorial_step, self.quiz_answer),
			Screen::Opponents => opponents::view(self.language, self.variant),
			Screen::Game => match &self.game {
				Some(game) => table::view(game, self.opponent, self.language, self.hints),
				None => menu::view(self.language, self.variant, false),
			},
		};
		let background = canvas(scene::Wall).width(Fill).height(Fill);
		stack![
			background,
			container(content)
				.width(Fill)
				.height(Fill)
				.style(widgets::ambient),
		]
		.into()
	}
}
