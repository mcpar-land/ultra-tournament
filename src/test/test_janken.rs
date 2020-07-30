use crate::tournament::*;
use rand::prelude::*;
use std::fmt;

#[derive(Debug)]
struct JankenFighter(f32, f32, f32);

#[derive(Eq, PartialEq)]
pub enum JankenRoll {
	Rock,
	Paper,
	Scissors,
}

impl fmt::Display for JankenFighter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Rock: {}% - Paper: {}% - Scissors: {}%",
			self.0 * 100.0,
			self.1 * 100.0,
			self.2 * 100.0
		)
	}
}

impl JankenFighter {
	fn roll(&self) -> JankenRoll {
		let rock_roll = random::<f32>() * self.0;
		let paper_roll = random::<f32>() * self.1;
		let scissors_roll = random::<f32>() * self.2;
		let mut res = JankenRoll::Rock;
		if paper_roll > rock_roll {
			res = JankenRoll::Paper;
		}
		if (res == JankenRoll::Rock && scissors_roll > rock_roll)
			|| (res == JankenRoll::Paper && scissors_roll > paper_roll)
		{
			res = JankenRoll::Scissors;
		}
		res
	}
}

impl Entrant for JankenFighter {
	fn battle(&mut self, enemy: &mut JankenFighter) -> TournamentRoundResult {
		use JankenRoll::*;
		use TournamentRoundResult::*;
		let my_roll = self.roll();
		let enemy_roll = enemy.roll();

		match my_roll {
			Rock => match enemy_roll {
				Rock => Tie,
				Paper => WinB,
				Scissors => WinA,
			},
			Paper => match enemy_roll {
				Rock => WinA,
				Paper => Tie,
				Scissors => WinB,
			},
			Scissors => match enemy_roll {
				Rock => WinB,
				Paper => WinA,
				Scissors => Tie,
			},
		}
	}
}
