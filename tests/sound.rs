use super::*;
use crate::game::Player;
use crate::game::card::full_deck;

const ALL: [Sound; 2] = [Sound::Play, Sound::Knock];

#[test]
fn every_sound_is_audible_and_never_clips() {
	let mut rng = fastrand::Rng::with_seed(7);
	for sound in ALL {
		let samples = synthesize(sound, &mut rng);
		assert!(!samples.is_empty(), "{sound:?} is empty");
		let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
		assert!(peak > 0.05, "{sound:?} is too quiet: {peak}");
		assert!(peak <= 1.0, "{sound:?} clips: {peak}");
		assert!(samples.iter().all(|s| s.is_finite()));
	}
}

#[test]
fn sounds_are_short() {
	let mut rng = fastrand::Rng::with_seed(7);
	for sound in ALL {
		let seconds = synthesize(sound, &mut rng).len() as f32 / RATE as f32;
		assert!(seconds < 1.5, "{sound:?} lasts {seconds}s");
	}
}

#[test]
fn table_events_make_sounds() {
	let card = full_deck().into_iter().next().expect("a card");
	let played = Event::Played {
		player: Player::Human,
		card,
	};
	assert_eq!(Sound::for_event(&played), Some(Sound::Play));
	assert_eq!(Sound::for_event(&Event::HandDrawn), None);
	assert_eq!(Sound::for_event(&Event::TrickTied), None);
}
