//! Truco Paulista rules engine for a one-versus-one match.

use super::card::Card;
use super::card::shuffled_deck;

/// Points needed to win a match.
pub const WINNING_SCORE: u8 = 12;
/// Score at which the "mão de onze" rule kicks in.
pub const ELEVEN: u8 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
	Human,
	Bot,
}

impl Player {
	pub fn opponent(self) -> Player {
		match self {
			Player::Human => Player::Bot,
			Player::Bot => Player::Human,
		}
	}

	pub fn index(self) -> usize {
		match self {
			Player::Human => 0,
			Player::Bot => 1,
		}
	}
}

/// How many points the current hand is worth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stake {
	One,
	Three,
	Six,
	Nine,
	Twelve,
}

impl Stake {
	pub fn value(self) -> u8 {
		match self {
			Stake::One => 1,
			Stake::Three => 3,
			Stake::Six => 6,
			Stake::Nine => 9,
			Stake::Twelve => 12,
		}
	}

	/// The stake a raise would lead to, if any.
	pub fn next(self) -> Option<Stake> {
		match self {
			Stake::One => Some(Stake::Three),
			Stake::Three => Some(Stake::Six),
			Stake::Six => Some(Stake::Nine),
			Stake::Nine => Some(Stake::Twelve),
			Stake::Twelve => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrickResult {
	Won(Player),
	Tied,
}

impl TrickResult {
	pub fn winner(self) -> Option<Player> {
		match self {
			TrickResult::Won(player) => Some(player),
			TrickResult::Tied => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
	/// A player with 11 points looks at their cards and decides whether to play.
	ElevenDecision { player: Player },
	/// The player whose turn it is may play a card or raise the stake.
	Playing,
	/// A raise was proposed and the responder must accept, raise again or fold.
	AwaitingResponse { responder: Player, proposed: Stake },
	/// The hand is finished; waiting for the next one to be dealt.
	HandOver,
	/// Someone reached 12 points.
	MatchOver { winner: Player },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
	/// Play the card at this index of the player's hand.
	Play(usize),
	/// Ask for truco (or six, nine, twelve).
	Raise,
	/// Accept a raise, or agree to play a "mão de onze".
	Accept,
	/// Run from a raise, or refuse to play a "mão de onze".
	Fold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandOutcome {
	Won { winner: Player, points: u8 },
	Draw,
}

/// Things that happened during the match, used to build the on-screen log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
	NewHand { starter: Player, vira: Card },
	ElevenHand { player: Player },
	IronHand,
	Played { player: Player, card: Card },
	TrickWon { player: Player, card: Card },
	TrickTied,
	Raised { player: Player, stake: Stake },
	Accepted { player: Player, stake: Stake },
	Folded { player: Player },
	ElevenAccepted { player: Player },
	ElevenFolded { player: Player },
	HandWon { player: Player, points: u8 },
	HandDrawn,
	MatchWon { player: Player },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionError {
	NotYourTurn,
	NotAllowed,
	InvalidCard,
}

#[derive(Debug, Clone)]
pub struct Game {
	rng: fastrand::Rng,
	pub scores: [u8; 2],
	/// The player who leads the first trick of the current hand ("mão").
	pub hand_starter: Player,
	pub vira: Card,
	pub hands: [Vec<Card>; 2],
	pub tricks: Vec<TrickResult>,
	/// Cards on the table for the trick in progress, in play order.
	pub table: Vec<(Player, Card)>,
	/// The most recently completed trick, kept so the UI can show it.
	pub last_trick: Vec<(Player, Card)>,
	/// Every card played in the current hand, in order (the pile on the table).
	pub hand_plays: Vec<(Player, Card)>,
	pub turn: Player,
	pub stake: Stake,
	/// The last player who raised; they cannot raise again until the other does.
	pub last_raiser: Option<Player>,
	/// "Mão de onze" or "mão de ferro": raising is forbidden.
	pub eleven_hand: bool,
	pub phase: Phase,
	pub outcome: Option<HandOutcome>,
	pub events: Vec<Event>,
	pub hand_number: u32,
}

impl Game {
	pub fn new() -> Self {
		Self::with_rng(fastrand::Rng::new())
	}

	#[cfg(any(test, feature = "screenshots"))]
	pub fn with_seed(seed: u64) -> Self {
		Self::with_rng(fastrand::Rng::with_seed(seed))
	}

	fn with_rng(rng: fastrand::Rng) -> Self {
		let mut game = Self {
			rng,
			scores: [0, 0],
			hand_starter: Player::Human,
			vira: super::card::full_deck()[0],
			hands: [Vec::new(), Vec::new()],
			tricks: Vec::new(),
			table: Vec::new(),
			last_trick: Vec::new(),
			hand_plays: Vec::new(),
			turn: Player::Human,
			stake: Stake::One,
			last_raiser: None,
			eleven_hand: false,
			phase: Phase::Playing,
			outcome: None,
			events: Vec::new(),
			hand_number: 0,
		};
		game.start_hand();
		game
	}

	pub fn score(&self, player: Player) -> u8 {
		self.scores[player.index()]
	}

	pub fn hand(&self, player: Player) -> &[Card] {
		&self.hands[player.index()]
	}

	fn start_hand(&mut self) {
		let mut deck = shuffled_deck(&mut self.rng);
		self.hands[0] = deck.drain(..3).collect();
		self.hands[1] = deck.drain(..3).collect();
		self.vira = deck[0];
		self.tricks.clear();
		self.table.clear();
		self.last_trick.clear();
		self.hand_plays.clear();
		self.turn = self.hand_starter;
		self.stake = Stake::One;
		self.last_raiser = None;
		self.outcome = None;
		self.hand_number += 1;
		self.events.push(Event::NewHand {
			starter: self.hand_starter,
			vira: self.vira,
		});

		let human_eleven = self.score(Player::Human) == ELEVEN;
		let bot_eleven = self.score(Player::Bot) == ELEVEN;
		self.eleven_hand = human_eleven || bot_eleven;
		self.phase = match (human_eleven, bot_eleven) {
			(true, true) => {
				self.events.push(Event::IronHand);
				Phase::Playing
			}
			(true, false) => {
				self.events.push(Event::ElevenHand {
					player: Player::Human,
				});
				Phase::ElevenDecision {
					player: Player::Human,
				}
			}
			(false, true) => {
				self.events.push(Event::ElevenHand {
					player: Player::Bot,
				});
				Phase::ElevenDecision {
					player: Player::Bot,
				}
			}
			(false, false) => Phase::Playing,
		};
	}

	/// Deals the next hand after the previous one finished.
	pub fn next_hand(&mut self) {
		if self.phase == Phase::HandOver {
			self.hand_starter = self.hand_starter.opponent();
			self.start_hand();
		}
	}

	/// The player who must act right now, if any.
	pub fn actor(&self) -> Option<Player> {
		match self.phase {
			Phase::ElevenDecision { player } => Some(player),
			Phase::Playing => Some(self.turn),
			Phase::AwaitingResponse { responder, .. } => Some(responder),
			Phase::HandOver | Phase::MatchOver { .. } => None,
		}
	}

	/// The stake a raise by `player` would propose right now, if raising is allowed.
	pub fn raise_target(&self, player: Player) -> Option<Stake> {
		if self.eleven_hand {
			return None;
		}
		match self.phase {
			Phase::Playing if self.turn == player && self.last_raiser != Some(player) => {
				self.stake.next()
			}
			Phase::AwaitingResponse {
				responder,
				proposed,
			} if responder == player => proposed.next(),
			_ => None,
		}
	}

	pub fn is_legal(&self, player: Player, action: Action) -> bool {
		if self.actor() != Some(player) {
			return false;
		}
		match (self.phase, action) {
			(Phase::Playing, Action::Play(index)) => index < self.hand(player).len(),
			(Phase::Playing | Phase::AwaitingResponse { .. }, Action::Raise) => {
				self.raise_target(player).is_some()
			}
			(
				Phase::AwaitingResponse { .. } | Phase::ElevenDecision { .. },
				Action::Accept | Action::Fold,
			) => true,
			_ => false,
		}
	}

	/// All actions `player` may legally take right now.
	#[cfg(test)]
	pub fn legal_actions(&self, player: Player) -> Vec<Action> {
		let mut actions: Vec<Action> = (0..self.hand(player).len()).map(Action::Play).collect();
		actions.extend([Action::Raise, Action::Accept, Action::Fold]);
		actions.retain(|&action| self.is_legal(player, action));
		actions
	}

	pub fn apply(&mut self, player: Player, action: Action) -> Result<(), ActionError> {
		if self.actor() != Some(player) {
			return Err(ActionError::NotYourTurn);
		}
		if !self.is_legal(player, action) {
			return Err(match action {
				Action::Play(_) => ActionError::InvalidCard,
				_ => ActionError::NotAllowed,
			});
		}

		match (self.phase, action) {
			(Phase::Playing, Action::Play(index)) => self.play_card(player, index),
			(Phase::Playing, Action::Raise) => {
				let proposed = self.stake.next().ok_or(ActionError::NotAllowed)?;
				self.last_raiser = Some(player);
				self.events.push(Event::Raised {
					player,
					stake: proposed,
				});
				self.phase = Phase::AwaitingResponse {
					responder: player.opponent(),
					proposed,
				};
			}
			(Phase::AwaitingResponse { proposed, .. }, Action::Accept) => {
				self.stake = proposed;
				self.events.push(Event::Accepted {
					player,
					stake: proposed,
				});
				self.phase = Phase::Playing;
			}
			(Phase::AwaitingResponse { proposed, .. }, Action::Raise) => {
				// Raising back implicitly accepts the previous proposal.
				let counter = proposed.next().ok_or(ActionError::NotAllowed)?;
				self.stake = proposed;
				self.last_raiser = Some(player);
				self.events.push(Event::Raised {
					player,
					stake: counter,
				});
				self.phase = Phase::AwaitingResponse {
					responder: player.opponent(),
					proposed: counter,
				};
			}
			(Phase::AwaitingResponse { .. }, Action::Fold) => {
				self.events.push(Event::Folded { player });
				self.finish_hand(HandOutcome::Won {
					winner: player.opponent(),
					points: self.stake.value(),
				});
			}
			(Phase::ElevenDecision { .. }, Action::Accept) => {
				self.events.push(Event::ElevenAccepted { player });
				self.stake = Stake::Three;
				self.phase = Phase::Playing;
			}
			(Phase::ElevenDecision { .. }, Action::Fold) => {
				self.events.push(Event::ElevenFolded { player });
				self.finish_hand(HandOutcome::Won {
					winner: player.opponent(),
					points: 1,
				});
			}
			_ => return Err(ActionError::NotAllowed),
		}
		Ok(())
	}

	fn play_card(&mut self, player: Player, index: usize) {
		let card = self.hands[player.index()].remove(index);
		self.table.push((player, card));
		self.hand_plays.push((player, card));
		self.events.push(Event::Played { player, card });

		if self.table.len() < 2 {
			self.turn = player.opponent();
			return;
		}

		let (leader, lead_card) = self.table[0];
		let (follower, follow_card) = self.table[1];
		let lead_strength = lead_card.strength(self.vira);
		let follow_strength = follow_card.strength(self.vira);
		let result = match lead_strength.cmp(&follow_strength) {
			std::cmp::Ordering::Greater => TrickResult::Won(leader),
			std::cmp::Ordering::Less => TrickResult::Won(follower),
			std::cmp::Ordering::Equal => TrickResult::Tied,
		};
		match result {
			TrickResult::Won(winner) => {
				let card = if winner == leader {
					lead_card
				} else {
					follow_card
				};
				self.events.push(Event::TrickWon {
					player: winner,
					card,
				});
				self.turn = winner;
			}
			TrickResult::Tied => {
				self.events.push(Event::TrickTied);
				self.turn = leader;
			}
		}
		self.tricks.push(result);
		self.last_trick = std::mem::take(&mut self.table);

		if let Some(outcome) = decide_hand(&self.tricks) {
			let outcome = match outcome {
				HandOutcome::Won { winner, .. } => HandOutcome::Won {
					winner,
					points: self.stake.value(),
				},
				HandOutcome::Draw => HandOutcome::Draw,
			};
			self.finish_hand(outcome);
		}
	}

	fn finish_hand(&mut self, outcome: HandOutcome) {
		self.outcome = Some(outcome);
		match outcome {
			HandOutcome::Won { winner, points } => {
				let score = &mut self.scores[winner.index()];
				*score = (*score + points).min(WINNING_SCORE);
				self.events.push(Event::HandWon {
					player: winner,
					points,
				});
				if *score >= WINNING_SCORE {
					self.events.push(Event::MatchWon { player: winner });
					self.phase = Phase::MatchOver { winner };
					return;
				}
			}
			HandOutcome::Draw => self.events.push(Event::HandDrawn),
		}
		self.phase = Phase::HandOver;
	}

	/// Number of tricks each player has won in the current hand.
	pub fn tricks_won(&self, player: Player) -> usize {
		self
			.tricks
			.iter()
			.filter(|result| result.winner() == Some(player))
			.count()
	}
}

impl Default for Game {
	fn default() -> Self {
		Self::new()
	}
}

/// Decides the hand from the tricks played so far, using the Truco Paulista
/// tie rules:
///
/// - Two trick wins take the hand.
/// - If the first trick ties, the next decisive trick takes the hand.
/// - If a later trick ties, the winner of the first trick takes the hand.
/// - Three ties: nobody scores.
///
/// The returned `points` are a placeholder; the caller fills in the stake.
pub fn decide_hand(tricks: &[TrickResult]) -> Option<HandOutcome> {
	use TrickResult::Tied;
	use TrickResult::Won;

	let win = |winner| Some(HandOutcome::Won { winner, points: 0 });
	match tricks {
		[Won(a), Won(b)] if a == b => win(*a),
		[Won(a), Tied] | [Tied, Won(a)] => win(*a),
		[Won(a), Won(_), Tied] => win(*a),
		[_, _, Won(c)] => win(*c),
		[Tied, Tied, Tied] => Some(HandOutcome::Draw),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
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
		let mut game = Game::with_seed(1);
		game.hands = [human.to_vec(), bot.to_vec()];
		game.vira = vira;
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
		assert_eq!(game.stake, Stake::Three);
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
		let mut game = Game::with_seed(7);
		game.apply(Player::Human, Action::Raise).unwrap();
		game.apply(Player::Bot, Action::Raise).unwrap();
		assert_eq!(game.stake, Stake::Three);
		game.apply(Player::Human, Action::Fold).unwrap();
		assert_eq!(game.score(Player::Bot), 3);
		assert_eq!(game.phase, Phase::HandOver);
	}

	#[test]
	fn eleven_hand_forbids_raising_and_fold_gives_one_point() {
		let mut game = Game::with_seed(3);
		game.scores = [ELEVEN, 4];
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
	fn random_games_always_finish() {
		for seed in 0..200 {
			let mut game = Game::with_seed(seed);
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
			assert!(matches!(game.phase, Phase::MatchOver { .. }), "seed {seed}");
		}
	}
}
