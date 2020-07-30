use crate::*;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, Default)]
struct MyMetadata;
impl MyMetadata {
	pub fn new() -> Self {
		Self {}
	}
}
impl std::fmt::Display for MyMetadata {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "My Metadata")
	}
}

#[derive(Clone)]
struct U32BattleSystem;
impl BattleSystem<u32, MyMetadata> for U32BattleSystem {
	fn battle(
		a_arc: Arc<RwLock<u32>>,
		b_arc: Arc<RwLock<u32>>,
	) -> BattleResult<MyMetadata> {
		use TournamentRoundResult::*;
		let a = a_arc.read().unwrap();
		let b = b_arc.read().unwrap();

		if *a > *b {
			BattleResult::Solved(A, MyMetadata::new())
		} else if *a < *b {
			BattleResult::Solved(B, MyMetadata::new())
		} else {
			BattleResult::Tie
		}
	}
	fn tiebreaker(
		_: Arc<RwLock<u32>>,
		_: Arc<RwLock<u32>>,
	) -> (TournamentRoundResult, MyMetadata) {
		use rand::prelude::*;
		use TournamentRoundResult::*;
		(if random::<f32>() > 0.5 { A } else { B }, MyMetadata::new())
	}
}
