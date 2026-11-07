//! A heuristic truco opponent. The same logic powers the hints shown to the
//! player, so every decision carries a [`Reason`] that can be explained.

use crate::game::Action;
use crate::game::Card;
use crate::game::Game;
use crate::game::Phase;
use crate::game::Player;
use crate::game::Stake;

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

/// Chooses the best action for `player`. `bluff` adds randomness so the bot
/// is not fully predictable; hints pass `None` for deterministic advice.
pub fn decide(game: &Game, player: Player, rng: Option<&mut fastrand::Rng>) -> Option<Decision> {
	if game.actor() != Some(player) {
		return None;
	}
	let roll = rng.map_or(0.5, |rng| rng.f32());
	let conf = confidence(game, player);

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
			let can_raise = game.raise_target(player).is_some();
			let pressure = stake_pressure(proposed);
			if can_raise && conf > 0.78 + pressure && roll > 0.25 {
				Some(Decision {
					action: Action::Raise,
					reason: Reason::RaiseBack,
				})
			} else if conf > 0.42 + pressure || roll < 0.08 {
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
				// An occasional bluff keeps things interesting.
				let bluff = roll > 0.93 && game.tricks.len() < 2;
				if conf > threshold || bluff {
					return Some(Decision {
						action: Action::Raise,
						reason: Reason::RaiseStrong,
					});
				}
			}
			Some(choose_card(game, player))
		}
		Phase::HandOver | Phase::MatchOver { .. } => None,
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
