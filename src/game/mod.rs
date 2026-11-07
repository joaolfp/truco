//! Game rules: cards and the match engine.

pub mod card;
pub mod engine;
pub mod variant;

pub use card::Card;
pub use card::Rank;
pub use card::Suit;
pub use engine::Action;
pub use engine::Event;
pub use engine::Game;
pub use engine::HandOutcome;
pub use engine::Phase;
pub use engine::Player;
pub use engine::Stake;
pub use engine::TrickResult;
pub use variant::MINEIRO_MANILHAS;
pub use variant::Variant;
