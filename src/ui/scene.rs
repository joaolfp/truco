//! Canvas art for the "boteco" (neighborhood bar) look: the wooden wall behind
//! every screen, and the round wooden table where the cards pile up.

use std::f32::consts::PI;
use std::f32::consts::TAU;

use iced::Alignment;
use iced::Color;
use iced::Font;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::Vector;
use iced::alignment;
use iced::mouse;
use iced::widget::canvas::Frame;
use iced::widget::canvas::Geometry;
use iced::widget::canvas::Path;
use iced::widget::canvas::Program;
use iced::widget::canvas::Stroke;
use iced::widget::canvas::Text;
use iced::widget::canvas::gradient::Linear;
use iced::widget::canvas::path::Builder;

use super::widgets::BOLD;
use crate::game::Card;
use crate::game::Game;
use crate::game::Player;

/// Size of the cards drawn on the table.
const CARD_WIDTH: f32 = 84.0;
const CARD_HEIGHT: f32 = CARD_WIDTH * 1.45;

fn rgb(r: u8, g: u8, b: u8) -> Color {
	Color::from_rgb8(r, g, b)
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
	Color::from_rgba8(r, g, b, a)
}

/// Cheap deterministic noise in `0.0..1.0`, so drawings are stable between frames.
fn noise(seed: u32) -> f32 {
	let mut x = seed.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
	x ^= x >> 15;
	x = x.wrapping_mul(0x2C1B_3C6D);
	x ^= x >> 12;
	(x % 1000) as f32 / 1000.0
}

fn shade(color: Color, amount: f32) -> Color {
	Color::from_rgb(
		(color.r + amount).clamp(0.0, 1.0),
		(color.g + amount).clamp(0.0, 1.0),
		(color.b + amount).clamp(0.0, 1.0),
	)
}

/// The wooden wall drawn behind every screen.
pub struct Wall;

impl<Message> Program<Message> for Wall {
	type State = ();

	fn draw(
		&self,
		_state: &Self::State,
		renderer: &Renderer,
		_theme: &Theme,
		bounds: Rectangle,
		_cursor: mouse::Cursor,
	) -> Vec<Geometry> {
		let mut frame = Frame::new(renderer, bounds.size());
		draw_wall(&mut frame, bounds.size());
		vec![frame.into_geometry()]
	}
}

/// The round bar table with the deck, the vira, the pile of played cards and
/// a few props.
pub struct Table<'a> {
	pub game: &'a Game,
}

impl<Message> Program<Message> for Table<'_> {
	type State = ();

	fn draw(
		&self,
		_state: &Self::State,
		renderer: &Renderer,
		_theme: &Theme,
		bounds: Rectangle,
		_cursor: mouse::Cursor,
	) -> Vec<Geometry> {
		let mut frame = Frame::new(renderer, bounds.size());
		let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
		let radius = (bounds.width.min(bounds.height) / 2.0 - 6.0).max(120.0);

		draw_round_table(&mut frame, center, radius);
		draw_peanuts(
			&mut frame,
			Point::new(center.x - radius * 0.5, center.y + radius * 0.12),
			radius * 0.13,
		);
		draw_deck_and_vira(
			&mut frame,
			Point::new(center.x - radius * 0.6, center.y - radius * 0.3),
			self.game,
		);
		draw_pile(&mut frame, center, self.game);
		draw_beer(
			&mut frame,
			Point::new(center.x + radius * 0.48, center.y + radius * 0.1),
		);

		vec![frame.into_geometry()]
	}
}

/// Dark vertical wooden planks with warm light from above.
fn draw_wall(frame: &mut Frame, size: Size) {
	let base = rgb(0x3a, 0x22, 0x14);
	let plank = 84.0;
	let count = (size.width / plank).ceil() as u32 + 1;
	for i in 0..count {
		let x = i as f32 * plank;
		let color = shade(base, (noise(i) - 0.5) * 0.06);
		frame.fill_rectangle(Point::new(x, 0.0), Size::new(plank, size.height), color);
		frame.fill_rectangle(
			Point::new(x, 0.0),
			Size::new(2.0, size.height),
			rgba(0x18, 0x0d, 0x07, 0.8),
		);
		for j in 0..3 {
			let gx = x + 12.0 + noise(i * 7 + j) * (plank - 24.0);
			frame.fill_rectangle(
				Point::new(gx, 0.0),
				Size::new(1.0, size.height),
				rgba(0x22, 0x12, 0x08, 0.35),
			);
		}
	}

	let light = Linear::new(Point::ORIGIN, Point::new(0.0, size.height))
		.add_stop(0.0, rgba(0xff, 0xc8, 0x78, 0.12))
		.add_stop(0.5, rgba(0x00, 0x00, 0x00, 0.0))
		.add_stop(1.0, rgba(0x00, 0x00, 0x00, 0.35));
	frame.fill(&Path::rectangle(Point::ORIGIN, size), light);
}

/// The part of a circle between two horizontal lines, as a closed path.
fn circle_band(center: Point, radius: f32, y0: f32, y1: f32) -> Path {
	let half_width = |y: f32| {
		let dy = (y - center.y).clamp(-radius, radius);
		(radius * radius - dy * dy).max(0.0).sqrt()
	};
	let steps = 12;
	let mut builder = Builder::new();
	for s in 0..=steps {
		let y = y0 + (y1 - y0) * s as f32 / steps as f32;
		let point = Point::new(center.x - half_width(y), y);
		if s == 0 {
			builder.move_to(point);
		} else {
			builder.line_to(point);
		}
	}
	for s in (0..=steps).rev() {
		let y = y0 + (y1 - y0) * s as f32 / steps as f32;
		builder.line_to(Point::new(center.x + half_width(y), y));
	}
	builder.close();
	builder.build()
}

/// A round wooden table seen from above.
fn draw_round_table(frame: &mut Frame, center: Point, radius: f32) {
	// Shadow on the floor and the thick rim.
	frame.fill(
		&Path::circle(Point::new(center.x + 8.0, center.y + 12.0), radius),
		rgba(0, 0, 0, 0.5),
	);
	frame.fill(&Path::circle(center, radius), rgb(0x5c, 0x33, 0x16));

	let top = radius - 12.0;
	let base = rgb(0xa8, 0x6c, 0x3c);
	let plank_height = 58.0;
	let first = center.y - top;
	let planks = (2.0 * top / plank_height).ceil() as u32;

	for i in 0..planks {
		let y0 = first + i as f32 * plank_height;
		let y1 = (y0 + plank_height).min(center.y + top);
		let color = shade(base, (noise(100 + i) - 0.5) * 0.08);
		frame.fill(&circle_band(center, top, y0, y1), color);

		// Wavy grain lines across the plank.
		for j in 0..4 {
			let gy = y0 + 6.0 + noise(200 + i * 5 + j) * (y1 - y0 - 12.0).max(1.0);
			let dy = gy - center.y;
			if dy.abs() >= top - 4.0 {
				continue;
			}
			let half = (top * top - dy * dy).sqrt() - 4.0;
			let amplitude = 2.0 + noise(300 + i * 5 + j) * 3.0;
			let segments = ((2.0 * half) / 90.0).ceil().max(1.0) as u32;
			let step = 2.0 * half / segments as f32;
			let mut grain = Builder::new();
			grain.move_to(Point::new(center.x - half, gy));
			for s in 0..segments {
				let x0 = center.x - half + s as f32 * step;
				let dir = if s % 2 == 0 { 1.0 } else { -1.0 };
				grain.quadratic_curve_to(
					Point::new(x0 + step / 2.0, gy + amplitude * dir),
					Point::new(x0 + step, gy),
				);
			}
			frame.stroke(
				&grain.build(),
				Stroke::default()
					.with_width(1.2)
					.with_color(rgba(0x6b, 0x3e, 0x1c, 0.35)),
			);
		}

		// A knot here and there.
		if noise(400 + i) > 0.6 {
			let dy = (y0 + y1) / 2.0 - center.y;
			let half = (top * top - dy * dy).max(0.0).sqrt();
			if half > 60.0 {
				let kx = center.x - half + 30.0 + noise(500 + i) * (2.0 * half - 60.0);
				let knot = Point::new(kx, (y0 + y1) / 2.0);
				frame.fill(&Path::circle(knot, 5.0), rgba(0x5a, 0x30, 0x12, 0.6));
				frame.stroke(
					&Path::circle(knot, 10.0),
					Stroke::default()
						.with_width(1.0)
						.with_color(rgba(0x5a, 0x30, 0x12, 0.35)),
				);
			}
		}

		if i > 0 {
			let dy = y0 - center.y;
			let half = (top * top - dy * dy).max(0.0).sqrt();
			frame.fill_rectangle(
				Point::new(center.x - half, y0 - 1.0),
				Size::new(2.0 * half, 2.0),
				rgba(0x3d, 0x20, 0x0c, 0.7),
			);
		}
	}

	// Inner bevel and varnish highlight on the rim.
	frame.stroke(
		&Path::circle(center, top),
		Stroke::default()
			.with_width(3.0)
			.with_color(rgba(0x3d, 0x20, 0x0c, 0.8)),
	);
	frame.stroke(
		&Path::circle(center, radius - 2.0),
		Stroke::default()
			.with_width(2.0)
			.with_color(rgba(0xff, 0xd9, 0xa0, 0.25)),
	);
}

/// Text helper for card faces.
fn card_text(content: &str, position: Point, size: f32, color: Color, bold: bool) -> Text {
	Text {
		content: content.to_string(),
		position,
		color,
		size: size.into(),
		font: if bold { BOLD } else { Font::DEFAULT },
		align_x: Alignment::Center.into(),
		align_y: alignment::Vertical::Center,
		..Text::default()
	}
}

/// Draws a card centered at `center`, rotated by `angle` radians.
/// `None` draws the back of the card.
///
/// The angle is never exactly zero: rotated text is rendered as geometry, which
/// keeps the pile layered correctly (unrotated canvas text is always drawn on
/// top of everything).
fn draw_card(frame: &mut Frame, center: Point, angle: f32, card: Option<Card>, highlight: bool) {
	let angle = if angle.abs() < 0.01 { 0.01 } else { angle };
	let (w, h) = (CARD_WIDTH, CARD_HEIGHT);
	frame.with_save(|frame| {
		frame.translate(Vector::new(center.x, center.y));
		frame.rotate(angle);
		let top_left = Point::new(-w / 2.0, -h / 2.0);

		frame.fill(
			&Path::rounded_rectangle(
				Point::new(top_left.x + 3.0, top_left.y + 5.0),
				Size::new(w, h),
				8.0.into(),
			),
			rgba(0, 0, 0, 0.35),
		);
		let outline = Path::rounded_rectangle(top_left, Size::new(w, h), 8.0.into());

		match card {
			Some(card) => {
				frame.fill(&outline, rgb(0xfc, 0xfa, 0xf2));
				frame.stroke(
					&outline,
					Stroke::default()
						.with_width(if highlight { 4.0 } else { 1.0 })
						.with_color(if highlight {
							rgb(0xf9, 0xc7, 0x33)
						} else {
							rgb(0x99, 0x99, 0x99)
						}),
				);
				let color = if card.suit.is_red() {
					rgb(0xc7, 0x21, 0x21)
				} else {
					rgb(0x1a, 0x1a, 0x1e)
				};
				let corner_x = top_left.x + 14.0;
				frame.fill_text(card_text(
					card.rank.label(),
					Point::new(corner_x, top_left.y + 16.0),
					22.0,
					color,
					true,
				));
				frame.fill_text(card_text(
					card.suit.symbol(),
					Point::new(corner_x, top_left.y + 36.0),
					16.0,
					color,
					false,
				));
				frame.fill_text(card_text(
					card.suit.symbol(),
					Point::new(4.0, 12.0),
					46.0,
					color,
					false,
				));
			}
			None => {
				frame.fill(&outline, rgb(0x8c, 0x1a, 0x24));
				frame.stroke(
					&outline,
					Stroke::default().with_width(2.0).with_color(Color::WHITE),
				);
				frame.stroke(
					&Path::rounded_rectangle(
						Point::new(top_left.x + 7.0, top_left.y + 7.0),
						Size::new(w - 14.0, h - 14.0),
						5.0.into(),
					),
					Stroke::default()
						.with_width(2.0)
						.with_color(rgb(0xf9, 0xc7, 0x33)),
				);
			}
		}
	});
}

/// The remaining deck, face down, with the vira face up beside it (Paulista only).
fn draw_deck_and_vira(frame: &mut Frame, at: Point, game: &Game) {
	if let Some(vira) = game.vira {
		draw_card(
			frame,
			Point::new(at.x + 92.0, at.y + 22.0),
			0.16,
			Some(vira),
			false,
		);
	}
	for i in 0..6 {
		let offset = i as f32 * 1.4;
		draw_card(
			frame,
			Point::new(at.x + offset, at.y - offset),
			-0.06 + noise(1200 + i) * 0.04,
			None,
			false,
		);
	}
}

/// Every card played this hand, piled in the middle of the table. Each card is
/// tossed slightly toward its owner with a random twist, and later cards land
/// on top of earlier ones.
fn draw_pile(frame: &mut Frame, center: Point, game: &Game) {
	let seed = game.hand_number.wrapping_mul(97);
	for (index, &(player, card)) in game.hand_plays.iter().enumerate() {
		let i = index as u32;
		let toward = match player {
			Player::Human => 30.0,
			Player::Bot => -30.0,
		};
		let jitter_x = (noise(seed + i * 3) - 0.5) * 70.0;
		let jitter_y = (noise(seed + i * 3 + 1) - 0.5) * 30.0;
		let angle = (noise(seed + i * 3 + 2) - 0.5) * 0.9;
		draw_card(
			frame,
			Point::new(center.x + jitter_x, center.y + toward + jitter_y),
			angle,
			Some(card),
			game.is_manilha(card),
		);
	}
}

/// A brown long-neck beer bottle and a ribbed "copo americano" glass.
/// `base` is the point on the table where the bottle stands.
fn draw_beer(frame: &mut Frame, base: Point) {
	frame.fill(
		&Path::rounded_rectangle(
			Point::new(base.x - 6.0, base.y - 6.0),
			Size::new(140.0, 16.0),
			8.0.into(),
		),
		rgba(0, 0, 0, 0.3),
	);

	// Bottle.
	let body_w = 46.0;
	let body_h = 120.0;
	let neck_w = 18.0;
	let neck_h = 62.0;
	let bx = base.x;
	let body_top = base.y - body_h;

	let mut bottle = Builder::new();
	bottle.move_to(Point::new(bx, base.y));
	bottle.line_to(Point::new(bx, body_top + 14.0));
	bottle.quadratic_curve_to(
		Point::new(bx, body_top - 6.0),
		Point::new(bx + (body_w - neck_w) / 2.0, body_top - 24.0),
	);
	bottle.line_to(Point::new(
		bx + (body_w - neck_w) / 2.0,
		body_top - 24.0 - neck_h,
	));
	bottle.line_to(Point::new(
		bx + (body_w + neck_w) / 2.0,
		body_top - 24.0 - neck_h,
	));
	bottle.line_to(Point::new(bx + (body_w + neck_w) / 2.0, body_top - 24.0));
	bottle.quadratic_curve_to(
		Point::new(bx + body_w, body_top - 6.0),
		Point::new(bx + body_w, body_top + 14.0),
	);
	bottle.line_to(Point::new(bx + body_w, base.y));
	bottle.close();
	let bottle = bottle.build();
	frame.fill(&bottle, rgb(0x5b, 0x2c, 0x0b));
	frame.stroke(
		&bottle,
		Stroke::default()
			.with_width(1.5)
			.with_color(rgb(0x2e, 0x15, 0x05)),
	);

	let cap_x = bx + (body_w - neck_w) / 2.0 - 2.0;
	let cap_y = body_top - 24.0 - neck_h - 8.0;
	frame.fill(
		&Path::rounded_rectangle(
			Point::new(cap_x, cap_y),
			Size::new(neck_w + 4.0, 10.0),
			2.0.into(),
		),
		rgb(0xd4, 0xa0, 0x17),
	);
	frame.fill_rectangle(
		Point::new(bx, base.y - 86.0),
		Size::new(body_w, 46.0),
		rgb(0xf3, 0xe6, 0xc4),
	);
	frame.fill_rectangle(
		Point::new(bx, base.y - 72.0),
		Size::new(body_w, 14.0),
		rgb(0xc9, 0x3a, 0x2c),
	);
	frame.fill_rectangle(
		Point::new(bx + 7.0, body_top + 8.0),
		Size::new(5.0, body_h - 20.0),
		rgba(0xff, 0xff, 0xff, 0.18),
	);

	// Copo americano: slightly tapered, ribbed glass with beer and foam.
	let gx = bx + body_w + 22.0;
	let gw_top = 56.0;
	let gw_bottom = 44.0;
	let gh = 88.0;
	let inset = (gw_top - gw_bottom) / 2.0;
	let top_y = base.y - gh;

	let glass = |depth: f32| {
		let mut b = Builder::new();
		let y = top_y + depth;
		let t = depth / gh;
		b.move_to(Point::new(gx + inset, base.y));
		b.line_to(Point::new(gx + inset * t, y));
		b.line_to(Point::new(gx + gw_top - inset * t, y));
		b.line_to(Point::new(gx + gw_top - inset, base.y));
		b.close();
		b.build()
	};

	frame.fill(&glass(0.0), rgba(0xff, 0xff, 0xff, 0.22));
	frame.fill(&glass(16.0), rgb(0xf2, 0xa9, 0x1c));
	frame.fill(&glass(16.0), rgba(0xff, 0xd5, 0x4f, 0.35));
	let mut foam = Builder::new();
	foam.move_to(Point::new(gx + inset * 0.18, top_y + 16.0));
	foam.line_to(Point::new(gx + 1.0, top_y + 2.0));
	foam.line_to(Point::new(gx + gw_top - 1.0, top_y + 2.0));
	foam.line_to(Point::new(gx + gw_top - inset * 0.18, top_y + 16.0));
	foam.close();
	frame.fill(&foam.build(), rgb(0xfb, 0xf6, 0xe8));
	for (dx, r) in [(10.0, 6.0), (22.0, 7.0), (36.0, 6.5), (47.0, 5.0)] {
		frame.fill(
			&Path::circle(Point::new(gx + dx, top_y + 3.0), r),
			rgb(0xfb, 0xf6, 0xe8),
		);
	}
	for i in 0..7 {
		let x = gx + inset + 4.0 + noise(600 + i) * (gw_bottom - 8.0);
		let y = top_y + 24.0 + noise(700 + i) * (gh - 32.0);
		frame.fill(
			&Path::circle(Point::new(x, y), 1.6),
			rgba(0xff, 0xff, 0xff, 0.6),
		);
	}
	for i in 1..8 {
		let t = i as f32 / 8.0;
		let mut rib = Builder::new();
		rib.move_to(Point::new(gx + gw_top * t, top_y + 22.0));
		rib.line_to(Point::new(gx + inset + gw_bottom * t, base.y - 4.0));
		frame.stroke(
			&rib.build(),
			Stroke::default()
				.with_width(1.0)
				.with_color(rgba(0xff, 0xff, 0xff, 0.22)),
		);
	}
	frame.stroke(
		&glass(0.0),
		Stroke::default()
			.with_width(2.0)
			.with_color(rgba(0xff, 0xff, 0xff, 0.55)),
	);
}

/// A bowl of peanuts ("amendoim"), seen from above.
fn draw_peanuts(frame: &mut Frame, center: Point, radius: f32) {
	let radius = radius.clamp(36.0, 60.0);
	frame.fill(
		&Path::circle(Point::new(center.x + 5.0, center.y + 8.0), radius),
		rgba(0, 0, 0, 0.35),
	);
	frame.fill(&Path::circle(center, radius), rgb(0xf4, 0xf1, 0xea));
	frame.fill(&Path::circle(center, radius - 9.0), rgb(0xe7, 0xe0, 0xd2));

	// Peanuts spread evenly with a golden-angle spiral, slightly jittered.
	let count = 30;
	let golden = PI * (3.0 - 5.0_f32.sqrt());
	for i in 0..count {
		let t = (i as f32 + 0.5) / count as f32;
		let distance = t.sqrt() * (radius - 18.0);
		let angle = i as f32 * golden + noise(800 + i) * 0.4;
		let p = Point::new(
			center.x + angle.cos() * distance,
			center.y + angle.sin() * distance,
		);
		let color = shade(rgb(0xb5, 0x6a, 0x2c), (noise(1000 + i) - 0.5) * 0.12);
		let tilt = noise(1100 + i) * TAU;
		let (dx, dy) = (tilt.cos() * 3.0, tilt.sin() * 3.0);
		// Each peanut is two touching blobs.
		frame.fill(&Path::circle(Point::new(p.x - dx, p.y - dy), 4.6), color);
		frame.fill(&Path::circle(Point::new(p.x + dx, p.y + dy), 4.2), color);
		frame.fill(
			&Path::circle(Point::new(p.x - dx - 1.5, p.y - dy - 1.5), 1.6),
			rgba(0xff, 0xe0, 0xb0, 0.45),
		);
	}
	frame.stroke(
		&Path::circle(center, radius),
		Stroke::default()
			.with_width(2.0)
			.with_color(rgba(0xff, 0xff, 0xff, 0.8)),
	);
}
