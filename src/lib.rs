//! `ultra_tournament` is a crate for running single-elimination tournament brackets with arbitrary structs for the entrants and round computation.
//!
//! # Example
//! ```
//! use ultra_tournament::*;
//! use num_format::{Locale, ToFormattedString};
//! use rand::prelude::*;
//! use std::fmt;
//! use std::sync::{Arc, RwLock};
//!
//! #[derive(Debug, Clone, Copy)]
//! struct IntFighter(u32);
//! impl fmt::Display for IntFighter {
//! 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! 		write!(
//! 			f,
//! 			"Int Fighter: {}",
//! 			self.0.to_formatted_string(&Locale::en)
//! 		)
//! 	}
//! }
//!
//! #[derive(Clone)]
//! struct IntBattleSystem;
//!
//! impl BattleSystem<IntFighter, String> for IntBattleSystem {
//! 	fn battle(
//! 		a_arc: Arc<RwLock<IntFighter>>,
//! 		b_arc: Arc<RwLock<IntFighter>>,
//! 	) -> BattleResult<String> {
//! 		use TournamentRoundResult::*;
//! 		let a = *a_arc.read().unwrap();
//! 		let b = *b_arc.read().unwrap();
//! 		if a.0 == b.0 {
//! 			return BattleResult::Tie;
//! 		}
//!
//! 		let delta = (a.0 as i64 - b.0 as i64).abs();
//!
//! 		let (winner, winner_val) = if a.0 > b.0 { (A, a) } else { (B, b) };
//!
//! 		BattleResult::Solved(winner, format!("{} wins by {}!", winner_val, delta))
//! 	}
//! 	fn tiebreaker(
//! 		_: Arc<RwLock<IntFighter>>,
//! 		_: Arc<RwLock<IntFighter>>,
//! 	) -> (TournamentRoundResult, String) {
//! 		use TournamentRoundResult::*;
//! 		let res: f32 = random();
//! 		if res > 0.5 {
//! 			(A, "A won by random tiebreaker.".to_string())
//! 		} else {
//! 			(B, "B won by random tiebreaker.".to_string())
//! 		}
//! 	}
//! }
//! ```
#[warn(missing_docs)]
mod tournament;
#[warn(missing_docs)]
mod types;

#[doc(inline)]
pub use crate::tournament::*;
#[doc(inline)]
pub use crate::types::*;

#[cfg(test)]
mod test {
	mod test_docs;
	mod test_tournament;
}
