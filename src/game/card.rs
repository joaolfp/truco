//! Cards, suits, ranks and the 40-card truco deck.

use std::fmt;

/// Card suits, declared from weakest to strongest manilha order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
	/// Ouros ♦ — the weakest manilha.
	Diamonds,
	/// Espadas ♠ — known as "espadilha" when it is a manilha.
	Spades,
	/// Copas ♥ — known as "copas" when it is a manilha.
	Hearts,
	/// Paus ♣ — the strongest manilha, the famous "zap".
	Clubs,
}

impl Suit {
	pub const ALL: [Suit; 4] = [Suit::Diamonds, Suit::Spades, Suit::Hearts, Suit::Clubs];

	pub fn symbol(self) -> &'static str {
		match self {
			Suit::Diamonds => "♦",
			Suit::Spades => "♠",
			Suit::Hearts => "♥",
			Suit::Clubs => "♣",
		}
	}

	pub fn is_red(self) -> bool {
		matches!(self, Suit::Diamonds | Suit::Hearts)
	}

	/// Strength of this suit when the card is a manilha (0 = weakest).
	pub fn manilha_order(self) -> u8 {
		self as u8
	}
}

/// Card ranks, declared from weakest to strongest (8, 9 and 10 are not used).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
	Four,
	Five,
	Six,
	Seven,
	Queen,
	Jack,
	King,
	Ace,
	Two,
	Three,
}

impl Rank {
	pub const ALL: [Rank; 10] = [
		Rank::Four,
		Rank::Five,
		Rank::Six,
		Rank::Seven,
		Rank::Queen,
		Rank::Jack,
		Rank::King,
		Rank::Ace,
		Rank::Two,
		Rank::Three,
	];

	pub fn label(self) -> &'static str {
		match self {
			Rank::Four => "4",
			Rank::Five => "5",
			Rank::Six => "6",
			Rank::Seven => "7",
			Rank::Queen => "Q",
			Rank::Jack => "J",
			Rank::King => "K",
			Rank::Ace => "A",
			Rank::Two => "2",
			Rank::Three => "3",
		}
	}

	/// Base strength of the rank (0 = weakest).
	pub fn order(self) -> u8 {
		self as u8
	}

	/// The rank that follows this one, wrapping from 3 back to 4.
	///
	/// When this rank is the "vira", the next rank is the manilha.
	pub fn next(self) -> Rank {
		Rank::ALL[(self.order() as usize + 1) % Rank::ALL.len()]
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
	pub rank: Rank,
	pub suit: Suit,
}

impl Card {
	pub const fn new(rank: Rank, suit: Suit) -> Self {
		Self { rank, suit }
	}

	/// Whether this card is a manilha given the face-up "vira" card.
	pub fn is_manilha(self, vira: Card) -> bool {
		self.rank == vira.rank.next()
	}

	/// Absolute strength of this card for the current hand.
	///
	/// Regular cards range from 0 to 9 (suits do not matter, so equal ranks
	/// tie). Manilhas range from 10 to 13 and are ordered by suit.
	pub fn strength(self, vira: Card) -> u8 {
		if self.is_manilha(vira) {
			10 + self.suit.manilha_order()
		} else {
			self.rank.order()
		}
	}
}

impl fmt::Display for Card {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.rank.label(), self.suit.symbol())
	}
}

/// Builds the full 40-card deck in a fixed order.
pub fn full_deck() -> Vec<Card> {
	Rank::ALL
		.iter()
		.flat_map(|&rank| Suit::ALL.iter().map(move |&suit| Card::new(rank, suit)))
		.collect()
}

/// Builds a shuffled 40-card deck.
pub fn shuffled_deck(rng: &mut fastrand::Rng) -> Vec<Card> {
	let mut deck = full_deck();
	rng.shuffle(&mut deck);
	deck
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deck_has_forty_unique_cards() {
		let deck = full_deck();
		assert_eq!(deck.len(), 40);
		let unique: std::collections::HashSet<_> = deck.iter().collect();
		assert_eq!(unique.len(), 40);
	}

	#[test]
	fn manilha_wraps_from_three_to_four() {
		assert_eq!(Rank::Three.next(), Rank::Four);
		assert_eq!(Rank::Seven.next(), Rank::Queen);
	}

	#[test]
	fn manilha_beats_everything_and_zap_is_strongest() {
		let vira = Card::new(Rank::Seven, Suit::Hearts);
		let zap = Card::new(Rank::Queen, Suit::Clubs);
		let ouros = Card::new(Rank::Queen, Suit::Diamonds);
		let three = Card::new(Rank::Three, Suit::Clubs);
		assert!(zap.strength(vira) > ouros.strength(vira));
		assert!(ouros.strength(vira) > three.strength(vira));
	}

	#[test]
	fn regular_cards_of_same_rank_tie() {
		let vira = Card::new(Rank::Four, Suit::Hearts);
		let a = Card::new(Rank::Three, Suit::Clubs);
		let b = Card::new(Rank::Three, Suit::Diamonds);
		assert_eq!(a.strength(vira), b.strength(vira));
	}
}
