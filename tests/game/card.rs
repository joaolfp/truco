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
