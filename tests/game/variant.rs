use super::*;

#[test]
fn mineiro_manilhas_are_fixed_and_ordered() {
	let v = Variant::Mineiro;
	let zap = Card::new(Rank::Four, Suit::Clubs);
	let copas = Card::new(Rank::Seven, Suit::Hearts);
	let espadilha = Card::new(Rank::Ace, Suit::Spades);
	let pica_fumo = Card::new(Rank::Seven, Suit::Diamonds);
	let three = Card::new(Rank::Three, Suit::Hearts);
	let other_four = Card::new(Rank::Four, Suit::Hearts);
	assert!(v.strength(zap, None) > v.strength(copas, None));
	assert!(v.strength(copas, None) > v.strength(espadilha, None));
	assert!(v.strength(espadilha, None) > v.strength(pica_fumo, None));
	assert!(v.strength(pica_fumo, None) > v.strength(three, None));
	assert!(!v.is_manilha(other_four, None));
	assert_eq!(v.strength(other_four, None), 0);
}

#[test]
fn stake_ladders() {
	let ladder = [
		Stake::Base,
		Stake::Truco,
		Stake::Six,
		Stake::Nine,
		Stake::Twelve,
	];
	let paulista: Vec<u8> = ladder
		.iter()
		.map(|&s| Variant::Paulista.stake_value(s))
		.collect();
	let mineiro: Vec<u8> = ladder
		.iter()
		.map(|&s| Variant::Mineiro.stake_value(s))
		.collect();
	assert_eq!(paulista, [1, 3, 6, 9, 12]);
	assert_eq!(mineiro, [2, 4, 8, 10, 12]);
}
