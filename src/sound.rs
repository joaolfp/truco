//! Table sounds: cards hitting the felt and knocks for a truco.
//!
//! Every sound is synthesized at startup, so the game ships no audio files.
//! Playback runs on its own thread because the audio device handle is not
//! `Send` on every platform; if no device is available the game stays silent.

use std::f32::consts::TAU;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use rodio::Source;
use rodio::buffer::SamplesBuffer;

use crate::game::Event;

const RATE: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sound {
	/// A card slapped onto the table.
	Play,
	/// Knocking on the table to call truco (or six, nine, twelve).
	Knock,
}

impl Sound {
	/// The sound a game event makes, if any.
	pub fn for_event(event: &Event) -> Option<Sound> {
		match event {
			Event::Played { .. } => Some(Sound::Play),
			Event::Raised { .. } => Some(Sound::Knock),
			_ => None,
		}
	}
}

/// Sends sounds to the playback thread. Cheap to keep around.
pub struct Sounds {
	sender: Option<Sender<Sound>>,
}

impl Sounds {
	pub fn new() -> Self {
		let (sender, receiver) = mpsc::channel::<Sound>();
		let spawned = thread::Builder::new().name("sound".into()).spawn(move || {
			let Ok(mut sink) = rodio::DeviceSinkBuilder::open_default_sink() else {
				return;
			};

			sink.log_on_drop(false);
			let mut rng = fastrand::Rng::new();

			for sound in receiver {
				let samples = synthesize(sound, &mut rng);
				let buffer = SamplesBuffer::new(
					rodio::ChannelCount::MIN,
					rodio::SampleRate::new(RATE).expect("non-zero rate"),
					samples
						.into_iter()
						.map(|s| s as rodio::Sample)
						.collect::<Vec<_>>(),
				);
				sink.mixer().add(buffer.amplify(0.45));
			}
		});
		Self {
			sender: spawned.ok().map(|_| sender),
		}
	}

	pub fn play(&self, sound: Sound) {
		if let Some(sender) = &self.sender {
			let _ = sender.send(sound);
		}
	}

	/// Plays the sounds for the events at `events[from..]`.
	pub fn play_events(&self, events: &[Event], from: usize) {
		for event in events.iter().skip(from) {
			if let Some(sound) = Sound::for_event(event) {
				self.play(sound);
			}
		}
	}
}

/// Renders a sound as mono samples in `-1.0..=1.0`. A little randomness keeps
/// repeated sounds from feeling mechanical.
pub fn synthesize(sound: Sound, rng: &mut fastrand::Rng) -> Vec<f32> {
	let mut out = Vec::new();

	match sound {
		Sound::Play => {
			let tone = 0.9 + rng.f32() * 0.2;
			mix(&mut out, 0.0, &slap(rng, 0.05, 0.006 * tone, 0.8));
		}
		Sound::Knock => {
			mix(&mut out, 0.0, &knock(rng, 0.9));
			mix(&mut out, 0.16, &knock(rng, 1.0));
		}
	}

	normalize(&mut out);
	out
}

/// A card landing on the felt: a short, muffled tap. The noise is low-passed
/// twice so it reads as paper on cloth rather than hiss, and dies out within a
/// few milliseconds over a faint low thump.
fn slap(rng: &mut fastrand::Rng, length: f32, decay: f32, volume: f32) -> Vec<f32> {
	let n = samples(length);
	let mut low = 0.0;
	let mut lower = 0.0;
	let mut rumble = 0.0;
	(0..n)
		.map(|i| {
			let t = i as f32 / RATE as f32;
			let noise = rng.f32() * 2.0 - 1.0;
			// Two low-passes (~700 Hz) take the hiss off the top...
			low += 0.1 * (noise - low);
			lower += 0.1 * (low - lower);
			// ...and removing a very low band keeps it from sounding boomy.
			rumble += 0.01 * (lower - rumble);
			let tap = (lower - rumble) * 3.0;
			let thump = (TAU * 170.0 * t).sin() * (-t / 0.012).exp() * 0.45;
			let attack = (t / 0.0008).min(1.0);
			attack * ((-t / decay).exp() * tap + thump) * volume
		})
		.collect()
}

/// Knuckles on a wooden table: a damped low tone with a sharp click.
fn knock(rng: &mut fastrand::Rng, volume: f32) -> Vec<f32> {
	let pitch = 165.0 + rng.f32() * 20.0;
	let n = samples(0.18);
	(0..n)
		.map(|i| {
			let t = i as f32 / RATE as f32;
			let body = (TAU * pitch * t).sin() * (-t / 0.045).exp()
				+ 0.4 * (TAU * pitch * 2.7 * t).sin() * (-t / 0.02).exp();
			let click = (rng.f32() * 2.0 - 1.0) * (-t / 0.003).exp() * 0.6;
			(body + click) * volume
		})
		.collect()
}

fn samples(seconds: f32) -> usize {
	(seconds * RATE as f32) as usize
}

/// Adds `sound` into `out` starting at `at` seconds, growing `out` as needed.
fn mix(out: &mut Vec<f32>, at: f32, sound: &[f32]) {
	let start = samples(at);

	if out.len() < start + sound.len() {
		out.resize(start + sound.len(), 0.0);
	}

	for (o, s) in out[start..].iter_mut().zip(sound) {
		*o += s;
	}
}

/// Keeps the peak at or below full scale so overlapping layers never clip.
fn normalize(out: &mut [f32]) {
	let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));

	if peak > 1.0 {
		for s in out {
			*s /= peak;
		}
	}
}

#[cfg(test)]
#[path = "../tests/sound.rs"]
mod tests;
