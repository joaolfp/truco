//! Automated screenshot tour, enabled with `--features screenshots`.
//!
//! Walks through a few scenes, captures the window with iced's own screenshot
//! support and writes PNG files to `docs/screenshots/`, then exits.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Duration;

use iced::Task;
use iced::window;
use iced::window::Screenshot;

use super::App;
use super::Message;
use super::Screen;
use crate::bot;
use crate::game::Game;
use crate::game::Phase;
use crate::game::Player;
use crate::game::Variant;
use crate::i18n::Language;
use crate::opponents::Difficulty;
use crate::opponents::Opponent;

/// Time given to the window to render a scene before capturing it.
const SETTLE: Duration = Duration::from_millis(800);

const SCENES: [&str; 8] = [
	"menu",
	"opponents",
	"tutorial-manilhas",
	"tutorial-quiz",
	"game-hints",
	"game-truco",
	"game-english",
	"game-mineiro",
];

#[derive(Debug, Clone)]
pub enum Step {
	Setup(usize),
	Shoot(usize),
	Save(usize, Screenshot),
}

pub fn start() -> Task<Message> {
	wait(Step::Setup(0))
}

fn wait(step: Step) -> Task<Message> {
	Task::perform(
		async {
			std::thread::sleep(SETTLE);
		},
		move |()| Message::Capture(step.clone()),
	)
}

pub fn update(app: &mut App, step: Step) -> Task<Message> {
	match step {
		Step::Setup(scene) => {
			setup(app, scene);
			wait(Step::Shoot(scene))
		}
		Step::Shoot(scene) => window::latest()
			.and_then(window::screenshot)
			.map(move |shot| Message::Capture(Step::Save(scene, shot))),
		Step::Save(scene, shot) => {
			let path = output_dir().join(format!("{}.png", SCENES[scene]));

			if let Err(error) = save_png(&path, &shot) {
				eprintln!("failed to save {}: {error}", path.display());
			} else {
				println!("saved {}", path.display());
			}

			if scene + 1 < SCENES.len() {
				Task::done(Message::Capture(Step::Setup(scene + 1)))
			} else {
				iced::exit()
			}
		}
	}
}

fn setup(app: &mut App, scene: usize) {
	match scene {
		0 => {
			app.language = Language::PortugueseBr;
			app.screen = Screen::Menu;
		}
		1 => {
			app.screen = Screen::Opponents;
		}
		2 => {
			app.screen = Screen::Tutorial;
			app.tutorial_step = 3;
			app.quiz_answer = None;
		}
		3 => {
			app.tutorial_step = 5;
			app.quiz_answer = Some(1);
		}
		4 => {
			app.screen = Screen::Game;
			app.hints = true;
			app.opponent = Opponent::for_difficulty(Difficulty::Professional);
			app.game = Some(find_game(Variant::Paulista, |game| {
				game.phase == Phase::Playing
					&& game.turn == Player::Human
					&& game.table.len() == 1
					&& game.tricks.len() == 1
			}));
		}
		5 => {
			app.game = Some(find_game(Variant::Paulista, |game| {
				matches!(
					game.phase,
					Phase::AwaitingResponse {
						responder: Player::Human,
						..
					}
				)
			}));
		}
		6 => {
			app.language = Language::English;
			app.game = Some(find_game(Variant::Paulista, |game| {
				game.phase == Phase::Playing
					&& game.turn == Player::Human
					&& game.table.len() == 1
					&& game.tricks.is_empty()
			}));
		}
		_ => {
			app.language = Language::PortugueseBr;
			app.variant = Variant::Mineiro;
			app.opponent = Opponent::for_difficulty(Difficulty::Master);
			app.game = Some(find_game(Variant::Mineiro, |game| {
				game.phase == Phase::Playing
					&& game.turn == Player::Human
					&& game.table.len() == 1
					&& game.tricks.len() == 1
			}));
		}
	}
}

/// Plays deterministic games until one reaches a state matching `wanted`.
fn find_game(variant: Variant, wanted: impl Fn(&Game) -> bool) -> Game {
	for seed in 0..10_000 {
		let mut game = Game::with_seed(seed, variant);
		// Show a match in progress rather than a 0 x 0 scoreboard.
		game.scores = [5, 7];

		for _ in 0..12 {
			if wanted(&game) {
				return game;
			}

			let Some(actor) = game.actor() else { break };
			let Some(decision) = bot::decide(&game, actor, None) else {
				break;
			};

			if game.apply(actor, decision.action).is_err() {
				break;
			}
		}
	}

	Game::new(variant)
}

fn output_dir() -> PathBuf {
	let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/screenshots");
	let _ = std::fs::create_dir_all(&dir);
	dir
}

fn save_png(path: &PathBuf, shot: &Screenshot) -> Result<(), Box<dyn std::error::Error>> {
	let writer = BufWriter::new(File::create(path)?);
	let mut encoder = png::Encoder::new(writer, shot.size.width, shot.size.height);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	let mut writer = encoder.write_header()?;
	writer.write_image_data(&shot.rgba)?;
	Ok(())
}
