use super::*;
use crate::game::card::Rank;
use crate::game::card::Suit;

fn won(player: Player) -> TrickResult {
	TrickResult::Won(player)
}

fn winner_of(tricks: &[TrickResult]) -> Option<Option<Player>> {
	decide_hand(tricks).map(|outcome| match outcome {
		HandOutcome::Won { winner, .. } => Some(winner),
		HandOutcome::Draw => None,
	})
}

#[test]
fn tie_rules() {
	use Player::Bot;
	use Player::Human;
	use TrickResult::Tied;

	assert_eq!(winner_of(&[won(Human)]), None);
	assert_eq!(winner_of(&[won(Human), won(Human)]), Some(Some(Human)));
	assert_eq!(winner_of(&[won(Human), won(Bot)]), None);
	assert_eq!(winner_of(&[Tied, won(Bot)]), Some(Some(Bot)));
	assert_eq!(winner_of(&[won(Bot), Tied]), Some(Some(Bot)));
	assert_eq!(winner_of(&[Tied, Tied]), None);
	assert_eq!(winner_of(&[Tied, Tied, won(Human)]), Some(Some(Human)));
	assert_eq!(
		winner_of(&[won(Human), won(Bot), won(Bot)]),
		Some(Some(Bot))
	);
	assert_eq!(winner_of(&[won(Human), won(Bot), Tied]), Some(Some(Human)));
	assert_eq!(winner_of(&[Tied, Tied, Tied]), Some(None));
}

fn rigged(human: [Card; 3], bot: [Card; 3], vira: Card) -> Game {
	let mut game = Game::with_seed(1, Variant::Paulista);
	game.hands = [human.to_vec(), bot.to_vec()];
	game.vira = Some(vira);
	game
}

#[test]
fn full_hand_with_truco() {
	let c = Card::new;
	let mut game = rigged(
		[
			c(Rank::Three, Suit::Clubs),
			c(Rank::Two, Suit::Clubs),
			c(Rank::Four, Suit::Clubs),
		],
		[
			c(Rank::Four, Suit::Hearts),
			c(Rank::Five, Suit::Hearts),
			c(Rank::Six, Suit::Hearts),
		],
		c(Rank::Ace, Suit::Spades),
	);
	assert_eq!(game.actor(), Some(Player::Human));
	game.apply(Player::Human, Action::Raise).unwrap();
	assert_eq!(game.actor(), Some(Player::Bot));
	game.apply(Player::Bot, Action::Accept).unwrap();
	assert_eq!(game.stake, Stake::Truco);
	assert!(!game.is_legal(Player::Human, Action::Raise));

	game.apply(Player::Human, Action::Play(0)).unwrap();
	game.apply(Player::Bot, Action::Play(0)).unwrap();
	assert_eq!(game.turn, Player::Human);
	game.apply(Player::Human, Action::Play(0)).unwrap();
	game.apply(Player::Bot, Action::Play(0)).unwrap();
	assert_eq!(game.phase, Phase::HandOver);
	assert_eq!(game.score(Player::Human), 3);
}

#[test]
fn folding_awards_previous_stake() {
	let mut game = Game::with_seed(7, Variant::Paulista);
	game.apply(Player::Human, Action::Raise).unwrap();
	game.apply(Player::Bot, Action::Raise).unwrap();
	assert_eq!(game.stake, Stake::Truco);
	game.apply(Player::Human, Action::Fold).unwrap();
	assert_eq!(game.score(Player::Bot), 3);
	assert_eq!(game.phase, Phase::HandOver);
}

#[test]
fn eleven_hand_forbids_raising_and_fold_gives_one_point() {
	let mut game = Game::with_seed(3, Variant::Paulista);
	game.scores = [11, 4];
	game.phase = Phase::HandOver;
	game.next_hand();
	assert_eq!(
		game.phase,
		Phase::ElevenDecision {
			player: Player::Human
		}
	);
	game.apply(Player::Human, Action::Fold).unwrap();
	assert_eq!(game.score(Player::Bot), 5);
}

#[test]
fn mineiro_hand_of_ten() {
	let mut game = Game::with_seed(5, Variant::Mineiro);
	assert_eq!(game.vira, None);
	assert_eq!(game.stake_value(game.stake), 2);
	game.scores = [4, 10];
	game.phase = Phase::HandOver;
	game.next_hand();
	assert_eq!(
		game.phase,
		Phase::ElevenDecision {
			player: Player::Bot
		}
	);
	game.apply(Player::Bot, Action::Fold).unwrap();
	assert_eq!(game.score(Player::Human), 6);
}

#[test]
fn random_games_always_finish() {
	for variant in Variant::ALL {
		for seed in 0..200 {
			let mut game = Game::with_seed(seed, variant);
			let mut rng = fastrand::Rng::with_seed(seed);

			for _ in 0..10_000 {
				match game.phase {
					Phase::MatchOver { .. } => break,
					Phase::HandOver => game.next_hand(),
					_ => {
						let actor = game.actor().unwrap();
						let actions = game.legal_actions(actor);
						let action = actions[rng.usize(..actions.len())];
						game.apply(actor, action).unwrap();
					}
				}
			}

			assert!(
				matches!(game.phase, Phase::MatchOver { .. }),
				"{variant:?} seed {seed}"
			);
		}
	}
}
