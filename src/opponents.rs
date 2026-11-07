//! The fictional opponents you can challenge, one per difficulty level.

use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Difficulty {
	/// Plays almost at random and is easy to read.
	Beginner,
	/// Knows the basics but makes mistakes.
	Amateur,
	/// Solid heuristic play with occasional bluffs.
	#[default]
	Professional,
	/// Simulates the possible hands before every decision.
	Master,
}

impl Difficulty {
	pub const ALL: [Difficulty; 4] = [
		Difficulty::Beginner,
		Difficulty::Amateur,
		Difficulty::Professional,
		Difficulty::Master,
	];

	/// 1 to 4, used to draw the stars.
	pub fn level(self) -> usize {
		self as usize + 1
	}
}

/// A fictional opponent at the bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opponent {
	pub difficulty: Difficulty,
	pub name: &'static str,
	pub initials: &'static str,
	pub color: Color,
}

pub const OPPONENTS: [Opponent; 4] = [
	Opponent {
		difficulty: Difficulty::Beginner,
		name: "Tonho",
		initials: "T",
		color: Color::from_rgb(0.30, 0.65, 0.35),
	},
	Opponent {
		difficulty: Difficulty::Amateur,
		name: "Dona Cida",
		initials: "DC",
		color: Color::from_rgb(0.20, 0.50, 0.80),
	},
	Opponent {
		difficulty: Difficulty::Professional,
		name: "Seu Chico",
		initials: "SC",
		color: Color::from_rgb(0.80, 0.45, 0.10),
	},
	Opponent {
		difficulty: Difficulty::Master,
		name: "Bigode",
		initials: "B",
		color: Color::from_rgb(0.70, 0.15, 0.20),
	},
];

impl Opponent {
	pub fn for_difficulty(difficulty: Difficulty) -> Opponent {
		OPPONENTS
			.into_iter()
			.find(|opponent| opponent.difficulty == difficulty)
			.unwrap_or(OPPONENTS[2])
	}
}
