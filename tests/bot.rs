use super::*;
use crate::game::Rank;
use crate::game::Suit;
use crate::game::Variant;

#[test]
fn bot_beats_cheaply() {
	let mut game = Game::with_seed(11, Variant::Paulista);
	game.vira = Some(Card::new(Rank::Four, Suit::Clubs));
	game.hands[0] = vec![Card::new(Rank::King, Suit::Hearts)];
	game.hands[1] = vec![
		Card::new(Rank::Three, Suit::Clubs),
		Card::new(Rank::Ace, Suit::Clubs),
		Card::new(Rank::Six, Suit::Clubs),
	];
	game.hands[0].extend([
		Card::new(Rank::Six, Suit::Hearts),
		Card::new(Rank::Six, Suit::Spades),
	]);
	game.apply(Player::Human, Action::Play(0)).unwrap();
	let decision = choose_card(&game, Player::Bot);
	assert_eq!(decision.action, Action::Play(1));
	assert_eq!(decision.reason, Reason::BeatCheaply);
}

#[test]
fn every_difficulty_finishes_matches() {
	for (i, difficulty) in Difficulty::ALL.into_iter().enumerate() {
		for seed in 0..10 {
			let variant = Variant::ALL[seed as usize % 2];
			let mut game = Game::with_seed(seed + 1000 * i as u64, variant);
			let mut rng = fastrand::Rng::with_seed(seed);
			for _ in 0..5_000 {
				match game.phase {
					Phase::MatchOver { .. } => break,
					Phase::HandOver => game.next_hand(),
					_ => {
						let actor = game.actor().unwrap();
						let decision = decide_as(difficulty, &game, actor, &mut rng).unwrap();
						game.apply(actor, decision.action).unwrap();
					}
				}
			}
			assert!(
				matches!(game.phase, Phase::MatchOver { .. }),
				"{difficulty:?}"
			);
		}
	}
}

/// Plays a match with `first` as the human seat and `second` as the bot.
fn duel(first: Difficulty, second: Difficulty, seed: u64, variant: Variant) -> Player {
	let mut game = Game::with_seed(seed, variant);
	let mut rng = fastrand::Rng::with_seed(seed);
	loop {
		match game.phase {
			Phase::MatchOver { winner } => return winner,
			Phase::HandOver => game.next_hand(),
			_ => {
				let actor = game.actor().unwrap();
				let difficulty = if actor == Player::Human {
					first
				} else {
					second
				};
				let decision = decide_as(difficulty, &game, actor, &mut rng).unwrap();
				game.apply(actor, decision.action).unwrap();
			}
		}
	}
}

#[test]
fn harder_opponents_win_more() {
	let wins = |strong, weak| {
		(0..30)
			.filter(|&seed| {
				let variant = Variant::ALL[seed as usize % 2];
				duel(strong, weak, seed, variant) == Player::Human
			})
			.count()
	};
	let master_vs_beginner = wins(Difficulty::Master, Difficulty::Beginner);
	let pro_vs_beginner = wins(Difficulty::Professional, Difficulty::Beginner);
	let master_vs_pro = wins(Difficulty::Master, Difficulty::Professional);
	println!(
		"master/beginner {master_vs_beginner}/30, pro/beginner {pro_vs_beginner}/30, master/pro {master_vs_pro}/30"
	);
	assert!(master_vs_beginner > 20);
	assert!(pro_vs_beginner > 18);
	assert!(master_vs_pro >= 15);
}

#[test]
fn bot_plays_full_matches() {
	for seed in 0..200 {
		let variant = Variant::ALL[seed as usize % 2];
		let mut game = Game::with_seed(seed, variant);
		let mut rng = fastrand::Rng::with_seed(seed);
		for _ in 0..5_000 {
			match game.phase {
				Phase::MatchOver { .. } => break,
				Phase::HandOver => game.next_hand(),
				_ => {
					let actor = game.actor().unwrap();
					let decision = decide(&game, actor, Some(&mut rng)).unwrap();
					game.apply(actor, decision.action).unwrap();
				}
			}
		}
		assert!(matches!(game.phase, Phase::MatchOver { .. }), "seed {seed}");
	}
}
