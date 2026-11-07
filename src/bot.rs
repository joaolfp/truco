//! Truco opponents at four difficulty levels. The professional heuristic also
//! powers the hints shown to the player, so every decision carries a
//! [`Reason`] that can be explained.

use crate::game::Action;
use crate::game::Card;
use crate::game::Game;
use crate::game::HandOutcome;
use crate::game::Phase;
use crate::game::Player;
use crate::game::Stake;
use crate::opponents::Difficulty;

/// Why a move was chosen, so the UI can explain it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
	/// Beat the opponent's card spending as little as possible.
	BeatCheaply,
	/// Our card ties the trick, and a tie is enough to win the hand.
	TieIsEnough,
	/// No card can beat the opponent's, so throw away the weakest.
	DiscardWeakest,
	/// Opening the hand with a medium card to probe the opponent.
	LeadMedium,
	/// We already won a trick, so make the opponent spend a strong card.
	LeadWeakAfterWin,
	/// We must win this trick, so play the strongest card.
	LeadStrongest,
	/// Only one card left.
	LastCard,
	/// Strong hand: worth raising.
	RaiseStrong,
	/// Good hand: accept the raise.
	AcceptGood,
	/// Very strong hand: raise back.
	RaiseBack,
	/// Weak hand: run and lose only the current stake.
	FoldWeak,
	/// Good hand with 11 points: play it.
	ElevenPlay,
	/// Weak hand with 11 points: refuse and give away just one point.
	ElevenFold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
	pub action: Action,
	pub reason: Reason,
}

/// Normalized power of a card (0.0 weakest, 1.0 the zap).
fn power(game: &Game, card: Card) -> f32 {
	f32::from(game.strength(card)) / 13.0
}

/// Estimates how likely `player` is to win the current hand (0.0 – 1.0).
pub fn confidence(game: &Game, player: Player) -> f32 {
	let mut powers: Vec<f32> = game.hand(player).iter().map(|&c| power(game, c)).collect();
	powers.sort_by(|a, b| b.total_cmp(a));
	let p = |i: usize| powers.get(i).copied().unwrap_or(0.0);

	let wins = game.tricks_won(player);
	let losses = game.tricks_won(player.opponent());
	let first_tied = game.tricks.first().is_some_and(|t| t.winner().is_none());

	let mut estimate = match (wins, losses) {
		(0, 0) if first_tied => p(0),
		(0, 0) => 0.55 * p(0) + 0.3 * p(1) + 0.15 * p(2),
		(1, 0) => 0.45 + 0.55 * p(0),
		(0, 1) => 0.75 * p(0) * p(1).max(0.35),
		_ => p(0),
	};

	// If the opponent already played on this trick we know exactly what we face.
	if let Some(&(owner, card)) = game.table.first()
		&& owner != player
	{
		let to_beat = game.strength(card);
		let can_beat = game
			.hand(player)
			.iter()
			.any(|&c| game.strength(c) > to_beat);
		estimate += if can_beat { 0.15 } else { -0.2 };
	}

	estimate.clamp(0.0, 1.0)
}

fn sorted_indices(game: &Game, player: Player) -> Vec<usize> {
	let hand = game.hand(player);
	let mut indices: Vec<usize> = (0..hand.len()).collect();
	indices.sort_by_key(|&i| game.strength(hand[i]));
	indices
}

/// Picks which card to play.
fn choose_card(game: &Game, player: Player) -> Decision {
	let hand = game.hand(player);
	let by_strength = sorted_indices(game, player);
	let weakest = by_strength[0];
	let strongest = by_strength[by_strength.len() - 1];
	let play = |index, reason| Decision {
		action: Action::Play(index),
		reason,
	};

	if hand.len() == 1 {
		return play(0, Reason::LastCard);
	}

	if let Some(&(_, opponent_card)) = game.table.first() {
		let target = game.strength(opponent_card);
		if let Some(&index) = by_strength
			.iter()
			.find(|&&i| game.strength(hand[i]) > target)
		{
			return play(index, Reason::BeatCheaply);
		}
		// A tie wins the hand when we already took the first trick.
		let won_first = game.tricks.first().and_then(|t| t.winner()) == Some(player);
		if won_first
			&& let Some(&index) = by_strength
				.iter()
				.find(|&&i| game.strength(hand[i]) == target)
		{
			return play(index, Reason::TieIsEnough);
		}
		return play(weakest, Reason::DiscardWeakest);
	}

	match game.tricks.as_slice() {
		[] => play(by_strength[by_strength.len() / 2], Reason::LeadMedium),
		[first] if first.winner() == Some(player) => play(weakest, Reason::LeadWeakAfterWin),
		_ => play(strongest, Reason::LeadStrongest),
	}
}

/// How a heuristic player deviates from the "textbook" move.
#[derive(Debug, Clone, Copy)]
struct Style {
	/// Random error added to the confidence estimate (± this much).
	noise: f32,
	/// Chance of playing a random card instead of the sensible one.
	random_card: f32,
	/// Chance of raising with a hand that does not justify it.
	bluff: f32,
	/// Whether the player ever raises back after being raised.
	raises_back: bool,
}

const BEGINNER: Style = Style {
	noise: 0.35,
	random_card: 0.5,
	bluff: 0.0,
	raises_back: false,
};
const AMATEUR: Style = Style {
	noise: 0.15,
	random_card: 0.2,
	bluff: 0.03,
	raises_back: true,
};
const PROFESSIONAL: Style = Style {
	noise: 0.0,
	random_card: 0.0,
	bluff: 0.07,
	raises_back: true,
};

/// Chooses the textbook action for `player`. With `rng` set it plays like the
/// professional opponent (with bluffs); hints pass `None` for deterministic,
/// explainable advice.
pub fn decide(game: &Game, player: Player, rng: Option<&mut fastrand::Rng>) -> Option<Decision> {
	heuristic(game, player, PROFESSIONAL, rng)
}

/// Chooses an action for an opponent of the given difficulty.
pub fn decide_as(
	difficulty: Difficulty,
	game: &Game,
	player: Player,
	rng: &mut fastrand::Rng,
) -> Option<Decision> {
	match difficulty {
		Difficulty::Beginner => heuristic(game, player, BEGINNER, Some(rng)),
		Difficulty::Amateur => heuristic(game, player, AMATEUR, Some(rng)),
		Difficulty::Professional => heuristic(game, player, PROFESSIONAL, Some(rng)),
		Difficulty::Master => master::decide(game, player, rng),
	}
}

fn heuristic(
	game: &Game,
	player: Player,
	style: Style,
	mut rng: Option<&mut fastrand::Rng>,
) -> Option<Decision> {
	if game.actor() != Some(player) {
		return None;
	}
	let mut roll = || rng.as_mut().map_or(0.5, |rng| rng.f32());
	let conf = (confidence(game, player) + (roll() - 0.5) * 2.0 * style.noise).clamp(0.0, 1.0);

	match game.phase {
		Phase::ElevenDecision { .. } => Some(if conf >= 0.45 {
			Decision {
				action: Action::Accept,
				reason: Reason::ElevenPlay,
			}
		} else {
			Decision {
				action: Action::Fold,
				reason: Reason::ElevenFold,
			}
		}),
		Phase::AwaitingResponse { proposed, .. } => {
			let can_raise = style.raises_back && game.raise_target(player).is_some();
			let pressure = stake_pressure(proposed);
			if can_raise && conf > 0.78 + pressure && roll() > 0.25 {
				Some(Decision {
					action: Action::Raise,
					reason: Reason::RaiseBack,
				})
			} else if conf > 0.42 + pressure || roll() < 0.08 {
				Some(Decision {
					action: Action::Accept,
					reason: Reason::AcceptGood,
				})
			} else {
				Some(Decision {
					action: Action::Fold,
					reason: Reason::FoldWeak,
				})
			}
		}
		Phase::Playing => {
			if let Some(target) = game.raise_target(player) {
				let threshold = 0.68 + stake_pressure(target);
				let bluff = roll() < style.bluff && game.tricks.len() < 2;
				if conf > threshold || bluff {
					return Some(Decision {
						action: Action::Raise,
						reason: Reason::RaiseStrong,
					});
				}
			}
			let cards = game.hand(player).len();
			if cards > 1 && roll() < style.random_card {
				let index = ((roll() * cards as f32) as usize).min(cards - 1);
				return Some(Decision {
					action: Action::Play(index),
					reason: Reason::DiscardWeakest,
				});
			}
			Some(choose_card(game, player))
		}
		Phase::HandOver | Phase::MatchOver { .. } => None,
	}
}

/// The master: before each decision it imagines many possible opponent hands
/// (only from cards it has not seen, so it never cheats), plays them out and
/// picks the move with the best results.
mod master {
	use super::*;
	use crate::game::card::full_deck;

	/// Simulated deals per option.
	const SAMPLES: usize = 160;

	/// Plays the rest of the hand from `game` with a random guess of the
	/// opponent's cards. Returns 1.0 for a win, 0.5 for a draw and 0.0 for a loss.
	fn rollout(game: &Game, player: Player, first: Option<usize>, rng: &mut fastrand::Rng) -> f32 {
		let mut sim = game.clone();
		let opponent = player.opponent();

		let mut unseen: Vec<Card> = full_deck()
			.into_iter()
			.filter(|card| {
				!sim.hand(player).contains(card)
					&& Some(*card) != sim.vira
					&& !sim.hand_plays.iter().any(|(_, played)| played == card)
			})
			.collect();
		rng.shuffle(&mut unseen);
		let count = sim.hand(opponent).len();
		sim.hands[opponent.index()] = unseen.into_iter().take(count).collect();

		// Only card play is simulated; raises are decided outside.
		sim.phase = Phase::Playing;
		if let Some(index) = first
			&& sim.apply(player, Action::Play(index)).is_err()
		{
			return 0.0;
		}
		while sim.phase == Phase::Playing {
			let actor = sim.turn;
			let Action::Play(index) = choose_card(&sim, actor).action else {
				break;
			};
			if sim.apply(actor, Action::Play(index)).is_err() {
				break;
			}
		}
		match sim.outcome {
			Some(HandOutcome::Won { winner, .. }) if winner == player => 1.0,
			Some(HandOutcome::Draw) => 0.5,
			_ => 0.0,
		}
	}

	fn win_rate(game: &Game, player: Player, first: Option<usize>, rng: &mut fastrand::Rng) -> f32 {
		let total: f32 = (0..SAMPLES)
			.map(|_| rollout(game, player, first, rng))
			.sum();
		total / SAMPLES as f32
	}

	/// Best card to play and its estimated win rate. Among near-equal options,
	/// the weaker card is kept to save strength for later.
	fn best_card(game: &Game, player: Player, rng: &mut fastrand::Rng) -> (usize, f32) {
		let mut best = (0, -1.0);
		for index in sorted_indices(game, player) {
			let rate = win_rate(game, player, Some(index), rng);
			if rate > best.1 + 0.03 {
				best = (index, rate);
			}
		}
		best
	}

	pub fn decide(game: &Game, player: Player, rng: &mut fastrand::Rng) -> Option<Decision> {
		if game.actor() != Some(player) {
			return None;
		}
		let value = |stake| f32::from(game.stake_value(stake));

		match game.phase {
			Phase::ElevenDecision { .. } => {
				let p = win_rate(game, player, None, rng);
				// Expected points of playing versus the sure loss of running.
				let play = (2.0 * p - 1.0) * value(Stake::Truco);
				Some(if play > -value(Stake::Base) {
					Decision {
						action: Action::Accept,
						reason: Reason::ElevenPlay,
					}
				} else {
					Decision {
						action: Action::Fold,
						reason: Reason::ElevenFold,
					}
				})
			}
			Phase::AwaitingResponse { proposed, .. } => {
				let p = win_rate(game, player, None, rng);
				let accept = (2.0 * p - 1.0) * value(proposed);
				let fold = -value(game.stake);
				Some(if game.raise_target(player).is_some() && p > 0.82 {
					Decision {
						action: Action::Raise,
						reason: Reason::RaiseBack,
					}
				} else if accept > fold {
					Decision {
						action: Action::Accept,
						reason: Reason::AcceptGood,
					}
				} else {
					Decision {
						action: Action::Fold,
						reason: Reason::FoldWeak,
					}
				})
			}
			Phase::Playing => {
				let (index, p) = best_card(game, player, rng);
				if let Some(target) = game.raise_target(player) {
					let strong = p > 0.72 + stake_pressure(target);
					// Calculated bluffs: only with a playable hand, early on.
					let bluff = p > 0.4 && game.tricks.len() < 2 && rng.f32() < 0.1;
					if strong || bluff {
						return Some(Decision {
							action: Action::Raise,
							reason: Reason::RaiseStrong,
						});
					}
				}
				Some(Decision {
					action: Action::Play(index),
					reason: if game.hand(player).len() == 1 {
						Reason::LastCard
					} else {
						Reason::BeatCheaply
					},
				})
			}
			Phase::HandOver | Phase::MatchOver { .. } => None,
		}
	}
}

/// Higher stakes demand more confidence.
fn stake_pressure(stake: Stake) -> f32 {
	match stake {
		Stake::Base | Stake::Truco => 0.0,
		Stake::Six => 0.05,
		Stake::Nine => 0.1,
		Stake::Twelve => 0.15,
	}
}

#[cfg(test)]
mod tests {
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
}
