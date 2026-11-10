//! Rule differences between Truco Paulista and Truco Mineiro.

use super::card::Card;
use super::card::Rank;
use super::card::Suit;
use super::engine::Stake;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
	/// Manilhas change every hand with the "vira"; hands start at 1 point.
	#[default]
	Paulista,
	/// Fixed manilhas and no vira; hands start at 2 points.
	Mineiro,
}

/// Truco Mineiro's fixed manilhas, from weakest to strongest:
/// 7♦ (pica-fumo), A♠ (espadilha), 7♥ (copas) and 4♣ (zap).
pub const MINEIRO_MANILHAS: [Card; 4] = [
	Card::new(Rank::Seven, Suit::Diamonds),
	Card::new(Rank::Ace, Suit::Spades),
	Card::new(Rank::Seven, Suit::Hearts),
	Card::new(Rank::Four, Suit::Clubs),
];

impl Variant {
	pub const ALL: [Variant; 2] = [Variant::Paulista, Variant::Mineiro];

	/// Whether a "vira" card is turned up to define the manilhas.
	pub fn uses_vira(self) -> bool {
		self == Variant::Paulista
	}

	/// Absolute strength of a card for the current hand.
	///
	/// Regular cards range from 0 to 9 by rank (suits tie); manilhas range
	/// from 10 to 13.
	pub fn strength(self, card: Card, vira: Option<Card>) -> u8 {
		match (self, vira) {
			(Variant::Paulista, Some(vira)) => card.strength(vira),
			_ => MINEIRO_MANILHAS
				.iter()
				.position(|&manilha| manilha == card)
				.map_or(card.rank.order(), |index| 10 + index as u8),
		}
	}

	pub fn is_manilha(self, card: Card, vira: Option<Card>) -> bool {
		self.strength(card, vira) >= 10
	}

	/// Points a hand is worth at each level of the raise ladder.
	pub fn stake_value(self, stake: Stake) -> u8 {
		match (self, stake) {
			(Variant::Paulista, Stake::Base) => 1,
			(Variant::Paulista, Stake::Truco) => 3,
			(Variant::Paulista, Stake::Six) => 6,
			(Variant::Paulista, Stake::Nine) => 9,
			(Variant::Mineiro, Stake::Base) => 2,
			(Variant::Mineiro, Stake::Truco) => 4,
			(Variant::Mineiro, Stake::Six) => 8,
			(Variant::Mineiro, Stake::Nine) => 10,
			(_, Stake::Twelve) => 12,
		}
	}

	/// Score that triggers the special hand: "mão de onze" (Paulista) or
	/// "mão de dez" (Mineiro).
	pub fn special_hand_score(self) -> u8 {
		match self {
			Variant::Paulista => 11,
			Variant::Mineiro => 10,
		}
	}
}

#[cfg(test)]
#[path = "../../tests/game/variant.rs"]
mod tests;
