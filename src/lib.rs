// #![deny(warnings)]
#![warn(missing_docs)]
//! Connect 4 game crate
pub(crate) mod ai;
pub(crate) mod game;
pub use game::Game;
