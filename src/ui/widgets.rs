//! Reusable visual pieces: cards, panels, badges and colors.

use iced::Background;
use iced::Border;
use iced::Center;
use iced::Color;
use iced::Element;
use iced::Fill;
use iced::Font;
use iced::Shadow;
use iced::Vector;
use iced::font::Weight;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::space;
use iced::widget::text;

use crate::game::Card;
use crate::opponents::Opponent;

pub const GOLD: Color = Color::from_rgb(0.98, 0.78, 0.20);
/// Marks the card the hint suggests, distinct from the gold manilha border.
pub const HINT: Color = Color::from_rgb(0.30, 0.80, 0.95);
pub const RED_SUIT: Color = Color::from_rgb(0.78, 0.13, 0.13);
pub const BLACK_SUIT: Color = Color::from_rgb(0.10, 0.10, 0.12);
pub const HUMAN_COLOR: Color = Color::from_rgb(0.30, 0.75, 0.40);
pub const BOT_COLOR: Color = Color::from_rgb(0.90, 0.35, 0.30);
pub const MUTED: Color = Color::from_rgb(0.75, 0.82, 0.78);

pub const BOLD: Font = Font {
	weight: Weight::Bold,
	..Font::DEFAULT
};

/// Text color for screens drawn over the bar scene (no background of its own).
pub fn ambient(_theme: &iced::Theme) -> container::Style {
	container::Style {
		text_color: Some(Color::WHITE),
		..container::Style::default()
	}
}

/// A chalkboard with a wooden frame, like the scoreboards in a bar.
pub fn chalkboard(_theme: &iced::Theme) -> container::Style {
	container::Style {
		background: Some(Background::Color(Color::from_rgb(0.15, 0.18, 0.16))),
		border: Border::default()
			.rounded(6)
			.width(5)
			.color(Color::from_rgb(0.55, 0.35, 0.17)),
		shadow: Shadow {
			color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
			offset: Vector::new(0.0, 3.0),
			blur_radius: 8.0,
		},
		text_color: Some(Color::from_rgb(0.95, 0.95, 0.92)),
		..container::Style::default()
	}
}

/// A translucent dark panel used to group information.
pub fn panel(_theme: &iced::Theme) -> container::Style {
	container::Style {
		background: Some(Background::Color(Color::from_rgba(0.08, 0.04, 0.02, 0.72))),
		border: Border::default().rounded(12),
		text_color: Some(Color::WHITE),
		..container::Style::default()
	}
}

/// A highlighted panel for hints.
pub fn hint_panel(_theme: &iced::Theme) -> container::Style {
	container::Style {
		background: Some(Background::Color(Color { a: 0.15, ..HINT })),
		border: Border::default().rounded(12).width(2).color(HINT),
		text_color: Some(Color::WHITE),
		..container::Style::default()
	}
}

/// A small colored pill with text.
pub fn badge<'a, M: 'a>(label: impl text::IntoFragment<'a>, color: Color) -> Element<'a, M> {
	container(text(label).font(BOLD).size(16).color(Color::WHITE))
		.padding([4, 12])
		.style(move |_| container::Style {
			background: Some(Background::Color(color)),
			border: Border::default().rounded(20),
			..container::Style::default()
		})
		.into()
}

/// A round marker showing who won a round (or `None` when not played yet).
pub fn round_marker<'a, M: 'a>(fill: Option<Color>) -> Element<'a, M> {
	container(space())
		.width(22)
		.height(22)
		.style(move |_| container::Style {
			background: fill.map(Background::Color),
			border: Border::default().rounded(11).width(2).color(Color::WHITE),
			..container::Style::default()
		})
		.into()
}

/// Draws the face of a card. `highlight` adds a golden border (manilhas, hints).
pub fn card_face<'a, M: 'a>(card: Card, highlight: bool, width: f32) -> Element<'a, M> {
	card_face_bordered(card, highlight.then_some(GOLD), width)
}

/// Draws the face of a card, with a thick `border` in the given color if any.
pub fn card_face_bordered<'a, M: 'a>(
	card: Card,
	border: Option<Color>,
	width: f32,
) -> Element<'a, M> {
	let height = width * 1.45;
	let color = if card.suit.is_red() {
		RED_SUIT
	} else {
		BLACK_SUIT
	};

	let corner = column![
		text(card.rank.label())
			.font(BOLD)
			.size(width * 0.26)
			.color(color),
		text(card.suit.symbol()).size(width * 0.2).color(color),
	]
	.align_x(Center)
	.spacing(0);
	let center = container(text(card.suit.symbol()).size(width * 0.55).color(color)).center(Fill);

	container(column![corner, center])
		.width(width)
		.height(height)
		.padding(width * 0.07)
		.style(move |_| container::Style {
			background: Some(Background::Color(Color::from_rgb(0.99, 0.98, 0.95))),
			border: Border::default()
				.rounded(width * 0.1)
				.width(if border.is_some() { 4 } else { 1 })
				.color(border.unwrap_or(Color::from_rgb(0.6, 0.6, 0.6))),
			shadow: Shadow {
				color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
				offset: Vector::new(0.0, 3.0),
				blur_radius: 6.0,
			},
			..container::Style::default()
		})
		.into()
}

/// Draws the back of a card.
pub fn card_back<'a, M: 'a>(width: f32) -> Element<'a, M> {
	let height = width * 1.45;
	let inner = container(text("T").font(BOLD).size(width * 0.4).color(GOLD))
		.center(Fill)
		.style(|_| container::Style {
			border: Border::default().rounded(6).width(2).color(GOLD),
			..container::Style::default()
		});
	container(inner)
		.width(width)
		.height(height)
		.padding(width * 0.08)
		.style(move |_| container::Style {
			background: Some(Background::Color(Color::from_rgb(0.55, 0.10, 0.14))),
			border: Border::default()
				.rounded(width * 0.1)
				.width(2)
				.color(Color::WHITE),
			shadow: Shadow {
				color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
				offset: Vector::new(0.0, 3.0),
				blur_radius: 6.0,
			},
			..container::Style::default()
		})
		.into()
}

/// A clickable card. When `on_press` is `None` the card is shown as inactive.
pub fn card_button<'a, M: Clone + 'a>(
	card: Card,
	border: Option<Color>,
	width: f32,
	on_press: Option<M>,
) -> Element<'a, M> {
	button(card_face_bordered(card, border, width))
		.padding(4)
		.on_press_maybe(on_press)
		.style(|_, status| {
			let border = match status {
				button::Status::Hovered | button::Status::Pressed => {
					Border::default().rounded(12).width(3).color(Color::WHITE)
				}
				_ => Border::default().rounded(12),
			};

			button::Style {
				background: None,
				text_color: Color::WHITE,
				border,
				shadow: Shadow::default(),
				snap: false,
			}
		})
		.into()
}

/// A round portrait with the opponent's initials.
pub fn avatar<'a, M: 'a>(opponent: &Opponent, size: f32) -> Element<'a, M> {
	let color = opponent.color;
	container(
		text(opponent.initials)
			.font(BOLD)
			.size(size * 0.4)
			.color(Color::WHITE),
	)
	.center(size)
	.style(move |_| container::Style {
		background: Some(Background::Color(color)),
		border: Border::default()
			.rounded(size / 2.0)
			.width(3)
			.color(Color::WHITE),
		shadow: Shadow {
			color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
			offset: Vector::new(0.0, 2.0),
			blur_radius: 6.0,
		},
		..container::Style::default()
	})
	.into()
}
