use crate::*;
use num_format::{Locale, ToFormattedString};
use rand::prelude::*;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy)]
struct IntFighter(u32);
impl fmt::Display for IntFighter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Int Fighter: {}",
			self.0.to_formatted_string(&Locale::en)
		)
	}
}

#[derive(Clone)]
struct IntBattleSystem;

impl BattleSystem<IntFighter, String> for IntBattleSystem {
	fn battle(
		a_arc: Arc<RwLock<IntFighter>>,
		b_arc: Arc<RwLock<IntFighter>>,
	) -> BattleResult<String> {
		use TournamentRoundResult::*;
		let a = *a_arc.read().unwrap();
		let b = *b_arc.read().unwrap();
		if a.0 == b.0 {
			return BattleResult::Tie;
		}

		let delta = (a.0 as i64 - b.0 as i64).abs();

		let (winner, winner_val) = if a.0 > b.0 { (A, a) } else { (B, b) };

		BattleResult::Solved(winner, format!("{} wins by {}!", winner_val, delta))
	}
	fn tiebreaker(
		_: Arc<RwLock<IntFighter>>,
		_: Arc<RwLock<IntFighter>>,
	) -> (TournamentRoundResult, String) {
		use TournamentRoundResult::*;
		let res: f32 = random();
		if res > 0.5 {
			(A, "A won by random tiebreaker.".to_string())
		} else {
			(B, "B won by random tiebreaker.".to_string())
		}
	}
}

fn random_int_tournament(
	len: usize,
) -> Result<Tournament<IntFighter, String, IntBattleSystem>> {
	Tournament::<IntFighter, String, IntBattleSystem>::new_from_gen(len, || {
		let r = IntFighter(random::<u32>());
		r
	})
}

fn winner_127_tournament(
) -> Result<Tournament<IntFighter, String, IntBattleSystem>> {
	Tournament::<IntFighter, String, IntBattleSystem>::new(vec![
		IntFighter(6),
		IntFighter(1),
		IntFighter(2),
		IntFighter(9),
		IntFighter(3),
		IntFighter(4),
		IntFighter(127),
		IntFighter(5),
		IntFighter(8),
		IntFighter(7),
	])
}

#[test]
fn create_tournament() -> Result<()> {
	for i in 1..100 {
		println!("tournament size: {}", i);
		random_int_tournament(i)?;
	}
	Ok(())
}

#[test]
fn tournament_node_counts() -> Result<()> {
	println!("ENTRANTS, ROUNDS");
	for i in 1..200 {
		let t = random_int_tournament(i)?;
		let entrants = t
			.graph()
			.node_indices()
			.filter(|index| match t.graph().node_weight(*index).unwrap() {
				TournamentNode::Entrant(_) => true,
				_ => false,
			})
			.count();
		let rounds = t
			.graph()
			.node_indices()
			.filter(|index| match t.graph().node_weight(*index).unwrap() {
				TournamentNode::Round(_) => true,
				_ => false,
			})
			.count();
		// println!("COUNT: {}", i);
		// println!("{}", t);
		assert_eq!(entrants, i);
		assert_eq!(rounds, entrants - 1);
		// println!("\n=====================");
		// println!("{}, {}", entrants, (entrants as i32) - (rounds as i32));
	}
	Ok(())
}

#[test]
fn len_entrants() -> Result<()> {
	let t = random_int_tournament(100)?;
	assert_eq!(t.len_entrants(), 100);
	Ok(())
}

#[test]
fn len_rounds() -> Result<()> {
	let t = random_int_tournament(100)?;
	assert_eq!(t.len_rounds(), 99);
	Ok(())
}

#[test]
fn len_rounds_incomplete() -> Result<()> {
	let mut t = random_int_tournament(100)?;
	assert_eq!(t.len_rounds_incomplete(), 99);
	t.solve()?;
	assert_eq!(t.len_rounds_incomplete(), 0);
	Ok(())
}

#[test]
fn len_rounds_complete() -> Result<()> {
	let mut t = random_int_tournament(100)?;
	assert_eq!(t.len_rounds_complete(), 0);
	t.solve()?;
	assert_eq!(t.len_rounds_complete(), 99);
	Ok(())
}

#[test]
fn solve() -> Result<()> {
	let mut t = winner_127_tournament()?;
	t.solve()?;
	let winner = t.winner_entrant(*t.grand_finals())?.unwrap();
	let winner_read = winner.read().unwrap();
	assert_eq!(winner_read.0, 127);
	Ok(())
}

#[test]
fn metadata() -> Result<()> {
	let mut t = winner_127_tournament()?;
	t.solve()?;
	let meta = t
		.graph()
		.node_weight(*t.grand_finals())
		.unwrap()
		.metadata()
		.unwrap();

	assert_eq!(meta, &"Int Fighter: 127 wins by 118!".to_string());
	println!("{}", meta);
	Ok(())
}

#[test]
fn result_accessors() -> Result<()> {
	let mut t = winner_127_tournament()?;
	t.solve()?;
	let r = t.graph().node_weight(*t.grand_finals()).unwrap().result();
	assert_eq!(*r.unwrap(), TournamentRoundResult::A);
	Ok(())
}

#[test]
fn print() -> Result<()> {
	let mut t = random_int_tournament(33)?;
	print_tournament(&t)?;
	println!("Solving...");
	t.solve().unwrap();
	println!("Solved!");
	print_tournament(&t)?;
	Ok(())
}
