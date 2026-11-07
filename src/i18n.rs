//! User-facing text in Brazilian Portuguese (default) and English.

use std::fmt;

use crate::bot::Reason;
use crate::game::Action;
use crate::game::Event;
use crate::game::Game;
use crate::game::Phase;
use crate::game::Player;
use crate::game::Stake;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
	#[default]
	PortugueseBr,
	English,
}

impl Language {
	pub const ALL: [Language; 2] = [Language::PortugueseBr, Language::English];

	pub fn strings(self) -> &'static Strings {
		match self {
			Language::PortugueseBr => &PT_BR,
			Language::English => &EN,
		}
	}

	fn pick(self, pt: &'static str, en: &'static str) -> &'static str {
		match self {
			Language::PortugueseBr => pt,
			Language::English => en,
		}
	}

	/// Name of the call that raises the hand to `stake`.
	pub fn raise_name(self, stake: Stake) -> &'static str {
		match stake {
			Stake::One => "-",
			Stake::Three => "Truco",
			Stake::Six => self.pick("Seis", "Six"),
			Stake::Nine => self.pick("Nove", "Nine"),
			Stake::Twelve => self.pick("Doze", "Twelve"),
		}
	}

	pub fn points(self, points: u8) -> String {
		match (self, points) {
			(Language::PortugueseBr, 1) => "1 ponto".into(),
			(Language::PortugueseBr, n) => format!("{n} pontos"),
			(Language::English, 1) => "1 point".into(),
			(Language::English, n) => format!("{n} points"),
		}
	}

	pub fn hand_worth(self, stake: Stake) -> String {
		match self {
			Language::PortugueseBr => format!("Valendo {}", self.points(stake.value())),
			Language::English => format!("Worth {}", self.points(stake.value())),
		}
	}

	pub fn tutorial_progress(self, step: usize, total: usize) -> String {
		match self {
			Language::PortugueseBr => format!("Passo {step} de {total}"),
			Language::English => format!("Step {step} of {total}"),
		}
	}

	pub fn bot_raised(self, stake: Stake) -> String {
		match self {
			Language::PortugueseBr => format!(
				"O bot pediu {}! Aceitar faz a mão valer {}.",
				self.raise_name(stake).to_uppercase(),
				self.points(stake.value())
			),
			Language::English => format!(
				"The bot called {}! Accepting makes the hand worth {}.",
				self.raise_name(stake).to_uppercase(),
				self.points(stake.value())
			),
		}
	}

	pub fn raise_button(self, stake: Stake) -> String {
		format!("{}!", self.raise_name(stake).to_uppercase())
	}

	pub fn event(self, event: &Event) -> String {
		let pt = self == Language::PortugueseBr;
		let human = |player: &Player| *player == Player::Human;
		match event {
			Event::NewHand { starter, vira } => match (pt, human(starter)) {
				(true, true) => format!("Nova mão! Vira: {vira}. Você começa."),
				(true, false) => format!("Nova mão! Vira: {vira}. O bot começa."),
				(false, true) => format!("New hand! Vira: {vira}. You lead."),
				(false, false) => format!("New hand! Vira: {vira}. The bot leads."),
			},
			Event::ElevenHand { player } => match (pt, human(player)) {
				(true, true) => "Mão de onze! Veja suas cartas e decida se joga.".into(),
				(true, false) => "Mão de onze do bot: ele decide se joga.".into(),
				(false, true) => "Hand of eleven! Look at your cards and decide whether to play.".into(),
				(false, false) => "The bot has a hand of eleven: it decides whether to play.".into(),
			},
			Event::IronHand => self
				.pick(
					"Mão de ferro! Os dois têm 11: vale 1 ponto e não tem truco.",
					"Iron hand! Both have 11: worth 1 point and no truco.",
				)
				.into(),
			Event::Played { player, card } => match (pt, human(player)) {
				(true, true) => format!("Você jogou {card}."),
				(true, false) => format!("O bot jogou {card}."),
				(false, true) => format!("You played {card}."),
				(false, false) => format!("The bot played {card}."),
			},
			Event::TrickWon { player, card } => match (pt, human(player)) {
				(true, true) => format!("Você ganhou a rodada com {card}."),
				(true, false) => format!("O bot ganhou a rodada com {card}."),
				(false, true) => format!("You won the round with {card}."),
				(false, false) => format!("The bot won the round with {card}."),
			},
			Event::TrickTied => self.pick("Rodada empatada (cangou)!", "Round tied!").into(),
			Event::Raised { player, stake } => {
				let call = self.raise_name(*stake).to_uppercase();
				match (pt, human(player)) {
					(true, true) => format!("Você pediu {call}!"),
					(true, false) => format!("O bot pediu {call}!"),
					(false, true) => format!("You called {call}!"),
					(false, false) => format!("The bot called {call}!"),
				}
			}
			Event::Accepted { player, stake } => {
				let points = self.points(stake.value());
				match (pt, human(player)) {
					(true, true) => format!("Você aceitou. A mão vale {points}."),
					(true, false) => format!("O bot aceitou. A mão vale {points}."),
					(false, true) => format!("You accepted. The hand is worth {points}."),
					(false, false) => format!("The bot accepted. The hand is worth {points}."),
				}
			}
			Event::Folded { player } => match (pt, human(player)) {
				(true, true) => "Você correu.".into(),
				(true, false) => "O bot correu!".into(),
				(false, true) => "You ran.".into(),
				(false, false) => "The bot ran!".into(),
			},
			Event::ElevenAccepted { player } => match (pt, human(player)) {
				(true, true) => "Você vai jogar a mão de onze (vale 3).".into(),
				(true, false) => "O bot vai jogar a mão de onze (vale 3).".into(),
				(false, true) => "You'll play the hand of eleven (worth 3).".into(),
				(false, false) => "The bot will play the hand of eleven (worth 3).".into(),
			},
			Event::ElevenFolded { player } => match (pt, human(player)) {
				(true, true) => "Você correu da mão de onze.".into(),
				(true, false) => "O bot correu da mão de onze.".into(),
				(false, true) => "You ran from the hand of eleven.".into(),
				(false, false) => "The bot ran from the hand of eleven.".into(),
			},
			Event::HandWon { player, points } => {
				let points = self.points(*points);
				match (pt, human(player)) {
					(true, true) => format!("Você ganhou {points}!"),
					(true, false) => format!("O bot ganhou {points}."),
					(false, true) => format!("You won {points}!"),
					(false, false) => format!("The bot won {points}."),
				}
			}
			Event::HandDrawn => self
				.pick(
					"Mão empatada: ninguém marca ponto.",
					"Hand drawn: nobody scores.",
				)
				.into(),
			Event::MatchWon { player } => match (pt, human(player)) {
				(true, true) => "Você venceu a partida!".into(),
				(true, false) => "O bot venceu a partida.".into(),
				(false, true) => "You won the match!".into(),
				(false, false) => "The bot won the match.".into(),
			},
		}
	}

	/// Short description of a suggested action, e.g. "Play 3♣".
	pub fn suggestion(self, action: Action, game: &Game) -> String {
		match action {
			Action::Play(index) => {
				let card = game.hand(Player::Human)[index];
				match self {
					Language::PortugueseBr => format!("Jogue o {card}"),
					Language::English => format!("Play {card}"),
				}
			}
			Action::Raise => {
				let call = game
					.raise_target(Player::Human)
					.map_or("-", |stake| self.raise_name(stake))
					.to_uppercase();
				match self {
					Language::PortugueseBr => format!("Peça {call}!"),
					Language::English => format!("Call {call}!"),
				}
			}
			Action::Accept => match game.phase {
				Phase::ElevenDecision { .. } => self.strings().play_eleven.into(),
				_ => self.strings().accept.into(),
			},
			Action::Fold => self.strings().run.into(),
		}
	}

	pub fn reason(self, reason: Reason) -> &'static str {
		match reason {
			Reason::BeatCheaply => self.pick(
				"Jogue a carta mais fraca que ainda vence a do adversário: assim você guarda as fortes.",
				"Play the weakest card that still beats your opponent's, saving the strong ones.",
			),
			Reason::TieIsEnough => self.pick(
				"Você ganhou a primeira rodada, então empatar esta já garante a mão.",
				"You won the first round, so tying this one already wins the hand.",
			),
			Reason::DiscardWeakest => self.pick(
				"Nenhuma carta sua vence a do adversário. Descarte a mais fraca.",
				"None of your cards beats your opponent's. Throw away the weakest one.",
			),
			Reason::LeadMedium => self.pick(
				"Abra a mão com uma carta média para sentir o jogo do adversário.",
				"Open the hand with a medium card to feel out your opponent.",
			),
			Reason::LeadWeakAfterWin => self.pick(
				"Você já ganhou uma rodada. Jogue fraco e obrigue o adversário a gastar uma carta forte.",
				"You already won a round. Play weak and force your opponent to spend a strong card.",
			),
			Reason::LeadStrongest => self.pick(
				"Você precisa ganhar esta rodada: jogue sua carta mais forte.",
				"You must win this round: play your strongest card.",
			),
			Reason::LastCard => self.pick("Só resta uma carta.", "Only one card left."),
			Reason::RaiseStrong => self.pick(
				"Sua mão está forte: peça truco para valer mais pontos!",
				"Your hand is strong: call truco to play for more points!",
			),
			Reason::AcceptGood => self.pick(
				"Sua mão tem chance: aceite.",
				"Your hand has a fair chance: accept.",
			),
			Reason::RaiseBack => self.pick(
				"Sua mão está muito forte: aumente a aposta!",
				"Your hand is very strong: raise again!",
			),
			Reason::FoldWeak => self.pick(
				"Sua mão está fraca: corra e perca só o valor atual.",
				"Your hand is weak: run and lose only the current value.",
			),
			Reason::ElevenPlay => self.pick(
				"Suas cartas são boas: jogue a mão de onze.",
				"Your cards are good: play the hand of eleven.",
			),
			Reason::ElevenFold => self.pick(
				"Suas cartas são fracas: correr custa só 1 ponto.",
				"Your cards are weak: running costs only 1 point.",
			),
		}
	}
}

impl fmt::Display for Language {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Language::PortugueseBr => "Português (BR)",
			Language::English => "English",
		})
	}
}

pub struct TutorialStep {
	pub title: &'static str,
	pub body: &'static str,
}

pub struct Strings {
	pub subtitle: &'static str,
	pub play: &'static str,
	pub continue_match: &'static str,
	pub tutorial: &'static str,
	pub language: &'static str,
	pub menu: &'static str,
	pub you: &'static str,
	pub bot: &'static str,
	pub vira: &'static str,
	pub manilha: &'static str,
	pub rounds: &'static str,
	pub log: &'static str,
	pub hints: &'static str,
	pub hint: &'static str,
	pub your_turn: &'static str,
	pub bot_thinking: &'static str,
	pub accept: &'static str,
	pub run: &'static str,
	pub play_eleven: &'static str,
	pub eleven_prompt: &'static str,
	pub next_hand: &'static str,
	pub new_match: &'static str,
	pub you_won_match: &'static str,
	pub bot_won_match: &'static str,
	pub previous: &'static str,
	pub next: &'static str,
	pub start_playing: &'static str,
	pub quiz_prompt: &'static str,
	pub quiz_correct: &'static str,
	pub quiz_wrong: &'static str,
	pub weakest: &'static str,
	pub strongest: &'static str,
	pub manilha_names: [&'static str; 4],
	pub tutorial_steps: [TutorialStep; 12],
}

pub static PT_BR: Strings = Strings {
	subtitle: "Truco Paulista • você contra o bot",
	play: "Jogar",
	continue_match: "Continuar partida",
	tutorial: "Tutorial",
	language: "Idioma",
	menu: "Menu",
	you: "Você",
	bot: "Bot",
	vira: "Vira",
	manilha: "Manilha",
	rounds: "Rodadas",
	log: "Histórico",
	hints: "Dicas",
	hint: "Dica",
	your_turn: "Sua vez! Escolha uma carta para jogar.",
	bot_thinking: "O bot está pensando...",
	accept: "Aceitar",
	run: "Correr",
	play_eleven: "Jogar a mão",
	eleven_prompt: "Mão de onze! Você tem 11 pontos. Jogar vale 3 pontos; correr dá 1 ponto ao bot.",
	next_hand: "Próxima mão",
	new_match: "Nova partida",
	you_won_match: "Parabéns, você venceu!",
	bot_won_match: "O bot venceu. Tente de novo!",
	previous: "Anterior",
	next: "Próximo",
	start_playing: "Jogar agora",
	quiz_prompt: "Clique na carta que vence:",
	quiz_correct: "Correto!",
	quiz_wrong: "Ops! Olhe a vira de novo e tente outra vez.",
	weakest: "mais fraca",
	strongest: "mais forte",
	manilha_names: [
		"Pica-fumo (ouros)",
		"Espadilha (espadas)",
		"Copas",
		"Zap (paus)",
	],
	tutorial_steps: [
		TutorialStep {
			title: "Bem-vindo ao Truco!",
			body: "Truco é um jogo de cartas de blefe e estratégia muito popular no Brasil. Aqui você joga o Truco Paulista, um contra um, contra o bot.\n\nO objetivo é ser o primeiro a fazer 12 pontos. Cada mão vale pontos, e você pode aumentar esse valor pedindo truco.",
		},
		TutorialStep {
			title: "O baralho",
			body: "O truco usa um baralho de 40 cartas: tiramos os 8, 9, 10 e os coringas.\n\nCada jogador recebe 3 cartas por mão. Depois, uma carta é virada para cima na mesa: é a vira.",
		},
		TutorialStep {
			title: "A força das cartas",
			body: "Da mais fraca para a mais forte, a ordem é:\n\n4, 5, 6, 7, Q (dama), J (valete), K (rei), A (ás), 2, 3.\n\nO naipe não importa nas cartas comuns: um 3 de ouros empata com um 3 de paus.",
		},
		TutorialStep {
			title: "A vira e as manilhas",
			body: "As manilhas são as cartas mais fortes do jogo. Elas são as cartas do valor logo acima da vira.\n\nExemplo: se a vira é um 7, as manilhas são as damas (Q). Se a vira é um 3, a sequência dá a volta e as manilhas são os 4.",
		},
		TutorialStep {
			title: "A ordem das manilhas",
			body: "Entre as manilhas, o naipe decide quem é mais forte:\n\nOuros (pica-fumo) < Espadas (espadilha) < Copas < Paus (zap).\n\nO zap é a carta mais forte de toda a mão!",
		},
		TutorialStep {
			title: "Teste: qual carta vence?",
			body: "A vira é o K de espadas. Pense: qual valor vem logo depois do K?",
		},
		TutorialStep {
			title: "As rodadas",
			body: "Cada mão tem até 3 rodadas. Em cada rodada, cada jogador joga uma carta e a mais forte vence.\n\nQuem ganhar 2 rodadas leva a mão. Quem vence uma rodada começa a próxima, e o jogador que começa a mão alterna a cada nova mão.",
		},
		TutorialStep {
			title: "Empates (cangou)",
			body: "Se duas cartas de mesma força se encontram, a rodada empata.\n\n• Empatou a 1ª: quem ganhar a próxima rodada leva a mão.\n• Empatou a 2ª ou a 3ª: quem ganhou a 1ª leva a mão.\n• Empataram as três: ninguém marca ponto.",
		},
		TutorialStep {
			title: "Pedindo truco",
			body: "Uma mão normal vale 1 ponto. Na sua vez, você pode pedir TRUCO para ela valer 3.\n\nO adversário pode:\n• Aceitar: a mão passa a valer 3.\n• Correr: você ganha o valor atual (1 ponto).\n• Aumentar: pedir SEIS. Depois vem NOVE e DOZE.\n\nQuem pediu por último não pode aumentar de novo: só o outro jogador.",
		},
		TutorialStep {
			title: "Teste: e agora?",
			body: "A vira é o 3 de ouros, então as manilhas são os 4. As duas cartas abaixo são manilhas: qual naipe é mais forte?",
		},
		TutorialStep {
			title: "Mão de onze e mão de ferro",
			body: "Quem chega a 11 pontos joga a mão de onze: vê as cartas e decide se joga (a mão vale 3) ou corre (o adversário ganha 1 ponto). Nessa mão ninguém pode pedir truco.\n\nSe os dois tiverem 11, é a mão de ferro: vale 1 ponto e também não tem truco.",
		},
		TutorialStep {
			title: "Pronto para jogar!",
			body: "Na partida, clique numa carta para jogá-la. Use os botões para pedir truco, aceitar ou correr.\n\nLigue as Dicas no topo da tela: elas sugerem uma jogada e explicam o porquê. Blefar faz parte do jogo, e o bot também blefa de vez em quando. Boa sorte!",
		},
	],
};

pub static EN: Strings = Strings {
	subtitle: "Truco Paulista • you versus the bot",
	play: "Play",
	continue_match: "Continue match",
	tutorial: "Tutorial",
	language: "Language",
	menu: "Menu",
	you: "You",
	bot: "Bot",
	vira: "Vira",
	manilha: "Manilha",
	rounds: "Rounds",
	log: "History",
	hints: "Hints",
	hint: "Hint",
	your_turn: "Your turn! Pick a card to play.",
	bot_thinking: "The bot is thinking...",
	accept: "Accept",
	run: "Run",
	play_eleven: "Play the hand",
	eleven_prompt: "Hand of eleven! You have 11 points. Playing is worth 3 points; running gives the bot 1 point.",
	next_hand: "Next hand",
	new_match: "New match",
	you_won_match: "Congratulations, you won!",
	bot_won_match: "The bot won. Try again!",
	previous: "Previous",
	next: "Next",
	start_playing: "Play now",
	quiz_prompt: "Click the winning card:",
	quiz_correct: "Correct!",
	quiz_wrong: "Oops! Look at the vira again and try once more.",
	weakest: "weakest",
	strongest: "strongest",
	manilha_names: ["Diamonds", "Spades", "Hearts", "Zap (clubs)"],
	tutorial_steps: [
		TutorialStep {
			title: "Welcome to Truco!",
			body: "Truco is a card game of bluffing and strategy, hugely popular in Brazil. Here you play Truco Paulista, one on one, against the bot.\n\nThe goal is to be the first to reach 12 points. Each hand is worth points, and you can raise that value by calling truco.",
		},
		TutorialStep {
			title: "The deck",
			body: "Truco uses a 40-card deck: the 8s, 9s, 10s and jokers are removed.\n\nEach player gets 3 cards per hand. Then one card is turned face up on the table: the vira.",
		},
		TutorialStep {
			title: "Card strength",
			body: "From weakest to strongest, the order is:\n\n4, 5, 6, 7, Q (queen), J (jack), K (king), A (ace), 2, 3.\n\nSuits don't matter for regular cards: a 3 of diamonds ties with a 3 of clubs.",
		},
		TutorialStep {
			title: "The vira and the manilhas",
			body: "Manilhas are the strongest cards in the game. They are the cards whose value comes right after the vira.\n\nExample: if the vira is a 7, the manilhas are the queens (Q). If the vira is a 3, the order wraps around and the manilhas are the 4s.",
		},
		TutorialStep {
			title: "Manilha order",
			body: "Among manilhas, the suit decides which is stronger:\n\nDiamonds < Spades < Hearts < Clubs (the zap).\n\nThe zap is the strongest card in the whole hand!",
		},
		TutorialStep {
			title: "Quiz: which card wins?",
			body: "The vira is the king of spades. Think: which value comes right after the K?",
		},
		TutorialStep {
			title: "Rounds",
			body: "Each hand has up to 3 rounds. In each round, each player plays one card and the strongest one wins.\n\nWhoever wins 2 rounds takes the hand. The winner of a round leads the next one, and the player who leads the hand alternates every new hand.",
		},
		TutorialStep {
			title: "Ties",
			body: "When two cards of the same strength meet, the round ties.\n\n• First round tied: whoever wins the next round takes the hand.\n• Second or third round tied: whoever won the first round takes the hand.\n• All three tied: nobody scores.",
		},
		TutorialStep {
			title: "Calling truco",
			body: "A normal hand is worth 1 point. On your turn, you can call TRUCO to make it worth 3.\n\nYour opponent can:\n• Accept: the hand is now worth 3.\n• Run: you win the current value (1 point).\n• Raise: call SIX. Then come NINE and TWELVE.\n\nWhoever raised last cannot raise again: only the other player can.",
		},
		TutorialStep {
			title: "Quiz: what now?",
			body: "The vira is the 3 of diamonds, so the manilhas are the 4s. Both cards below are manilhas: which suit is stronger?",
		},
		TutorialStep {
			title: "Hand of eleven and iron hand",
			body: "A player who reaches 11 points plays the hand of eleven: they see their cards and decide to play (the hand is worth 3) or run (the opponent gets 1 point). Nobody can call truco in this hand.\n\nIf both players have 11, it's the iron hand: worth 1 point and no truco either.",
		},
		TutorialStep {
			title: "Ready to play!",
			body: "During the match, click a card to play it. Use the buttons to call truco, accept or run.\n\nTurn on Hints at the top of the screen: they suggest a move and explain why. Bluffing is part of the game, and the bot bluffs now and then too. Good luck!",
		},
	],
};
