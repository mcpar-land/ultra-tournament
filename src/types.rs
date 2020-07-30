use petgraph::graph::NodeIndex;
use std::clone::Clone;
use std::default::Default;
use std::fmt;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};

/// Standard [`Result`](https://doc.rust-lang.org/std/result/) type alias for the library. Error type is [`TournamentError`](enum.TournamentError.html)
pub type Result<T> = std::result::Result<T, TournamentError>;

/// Implement this trait to create a system for solving battles betweeen two structs.
///
/// # Example
/// The larger number wins. Ties are resolved randomly.
/// ```
/// use crate::MyMetadata;
///
/// #[derive(Clone)]
/// struct U32BattleSystem;
/// impl BattleSystem<u32, MyMetadata> for U32BattleSystem {
///
/// 	fn battle(
/// 		a_arc: Arc<RwLock<u32>>,
/// 		b_arc: Arc<RwLock<u32>>,
/// 	) -> BattleResult<MyMetadata> {
/// 		use TournamentRoundResult::*;
/// 		let a = a_arc.read().unwrap();
/// 		let b = b_arc.read().unwrap();
///
/// 		if *a > *b {
/// 			BattleResult::Solved(A, MyMetadata::new())
/// 		} else if *a < *b {
/// 			BattleResult::Solved(B, MyMetadata::new())
/// 		} else {
/// 			BattleResult::Tie
/// 		}
/// 	}
///
/// 	fn tiebreaker(
/// 		_: Arc<RwLock<u32>>,
/// 		_: Arc<RwLock<u32>>,
/// 	) -> (TournamentRoundResult, MyMetadata) {
/// 		use rand::prelude::*;
/// 		use TournamentRoundResult::*;
/// 		(
/// 			if random::<f32>() > 0.5 { A } else { B },
/// 			MyMetadata::new()
/// 		)
/// 	}
///
/// }
/// ```
pub trait BattleSystem<
	E: Debug + Display + Clone,
	M: Debug + Display + Clone + Default,
>: Clone
{
	/// - Resolves a round played between two entrants encapsulated in [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)`<`[`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html)`<E>>`s, allowing for mutation of entrants between rounds.
	///
	/// - Example funcationality: reduce a fighter's HP during a round, and retain the change in later rounds.
	fn battle(a: Arc<RwLock<E>>, b: Arc<RwLock<E>>) -> BattleResult<M>;

	/// - In case `battle` returns a [`BattleResult::Tie`](enum.BattleResult.html#variant.Tie), run a tiebreaker that must return a successful result.
	fn tiebreaker(
		a: Arc<RwLock<E>>,
		b: Arc<RwLock<E>>,
	) -> (TournamentRoundResult, M);
}

/// Returned by the [`battle()`](trait.BattleSystem.html#tymethod.battle) function in implementations of [`BattleSystem`](trait.BattleSystem.html)
pub enum BattleResult<M: Debug + Display + Clone + Default> {
	/// A successful solve, returns whether [`A`](enum.TournamentRoundResult.html#variant.A) or [`B`](enum.TournamentRoundResult.html#variant.A) wins, along with a piece of round metadata of type `M`.
	Solved(TournamentRoundResult, M),
	/// A solve that resulted in a tie. When [`battle()`](trait.BattleSystem.html#tymethod.battle) returns this, [`tiebreaker()`](trait.BattleSystem.html#tymethod.tiebreaker) is run immediately after.
	Tie,
}

/// The Id of an entrant in a [`Tournament`](struct.Tournament.html). A wrapper around a single `usize`. Implements [`Display`](https://doc.rust-lang.org/stable/rust-by-example/hello/print/print_display.html)
#[derive(Debug, Clone, Copy)]
pub struct EntrantId(pub usize);
impl fmt::Display for EntrantId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Entrant #{}", self.0)
	}
}

/// The [node weight](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.Graph.html#method.node_weight) of a [`Tournament`](struct.Tournament.html)'s internal [graph](struct.Tournament.html#method.graph).
#[derive(Debug, Clone, Copy)]
pub enum TournamentNode<M: Debug + Display + Clone + Default> {
	/// Represents the starting point of an entrant within the tournament bracket. Links to exactly one `Round` node.
	Entrant(EntrantId),
	/// Represents a round in the tournament. Links to two previous rounds or entrant nodes, and one future round node (except for the final round)
	Round(TournamentRound<M>),
}
impl<M: Debug + Display + Clone + Default> fmt::Display for TournamentNode<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Entrant(eid) => write!(f, "{}", eid),
			Self::Round(r) => write!(f, "{}", r),
		}
	}
}
impl<M: Debug + Display + Clone + Default> TournamentNode<M> {
	/// Get the entrant of the node. Returns `None` if the node is a `TournamentNode::Round`
	pub fn entrant(&self) -> Option<&EntrantId> {
		match self {
			Self::Entrant(eid) => Some(eid),
			_ => None,
		}
	}
	/// Get the round of the node. Returns `None` if the node is a `TournamentNode::Entrant`
	pub fn round(&self) -> Option<&TournamentRound<M>> {
		match self {
			Self::Round(r) => Some(r),
			_ => None,
		}
	}
	/// Get the metadata of a node. Returns `None` if the node is a `TournamentNode::Entrant`, or is incomplete.
	pub fn metadata(&self) -> Option<&M> {
		if let Self::Round(round) = self {
			round.metadata()
		} else {
			None
		}
	}
	/// Get a mutable reference to the metadata of a node. Returns `None` if the node is a `TournamentNode::Entrant`, or is incomplete.
	pub fn metadata_mut(&mut self) -> Option<&mut M> {
		if let Self::Round(round) = self {
			round.metadata_mut()
		} else {
			None
		}
	}
	/// Get the result of a node. Returns `None` if the node is a `TournamentNode::Entrant`, or is incomplete.
	pub fn result(&self) -> Option<&TournamentRoundResult> {
		if let Self::Round(round) = self {
			round.result()
		} else {
			None
		}
	}
}

/// A single round in a [`Tournament`](struct.Tournament.html)'s bracket.
#[derive(Debug, Clone, Copy)]
pub enum TournamentRound<M: Debug + Display + Clone + Default> {
	/// Represents a round that hasn't be solved / played out yet.
	Incomplete,
	/// Represents a round that's been solved, and has a winner.
	Complete {
		/// The winner of the round.
		result: TournamentRoundResult,
		/// Metadata associated with this round, as returned from [`BattleSystem::battle`](trait.BattleSystem.html#tymethod.battle) or [`BattleSystem::tiebreaker`](trait.BattleSystem.html#tymethod.tiebreaker)
		metadata: M,
	},
}
impl<M: Debug + Display + Clone + Default> fmt::Display for TournamentRound<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Incomplete => write!(f, "Incomplete"),
			Self::Complete { result, metadata } => {
				write!(f, "{} --- {}", result, metadata)
			}
		}
	}
}
impl<M: Debug + Display + Clone + Default> TournamentRound<M> {
	/// Get the metadata of a round. Returns `None` if the round is incomplete.
	pub fn metadata(&self) -> Option<&M> {
		if let TournamentRound::<M>::Complete {
			result: _,
			metadata,
		} = self
		{
			Some(&metadata)
		} else {
			None
		}
	}
	/// Get a mutable reference to the metadata of a round. Returns `None` if the round is incomplete.
	pub fn metadata_mut(&mut self) -> Option<&mut M> {
		if let TournamentRound::<M>::Complete {
			result: _,
			metadata,
		} = self
		{
			Some(metadata)
		} else {
			None
		}
	}
	/// Get the result of a round. Returns `None` if the round is incomplete.
	pub fn result(&self) -> Option<&TournamentRoundResult> {
		if let TournamentRound::<M>::Complete {
			result,
			metadata: _,
		} = self
		{
			Some(result)
		} else {
			None
		}
	}
}

/// The [edge weight](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.Graph.html#method.edge_weight) of a [`Tournament`](struct.Tournament.html)'s internal [graph](struct.Tournament.html#method.graph).
///
/// Convertible to [`TournamentRoundResult`](enum.TournamentRoundResult.html)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TournamentEdge {
	/// Represents a connection from one round to the next on size `A`.
	A,
	/// Represents a connection from one round to the next on side `B`.
	B,
}
impl std::convert::From<TournamentRoundResult> for TournamentEdge {
	fn from(r: TournamentRoundResult) -> Self {
		match r {
			TournamentRoundResult::A => Self::A,
			TournamentRoundResult::B => Self::B,
		}
	}
}

/// Represents the winner of a solved [`TournamentRound`](enum.TournamentRound.html)
///
/// Convertible to [`TournamentEdge`](enum.TournamentEdge.html)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TournamentRoundResult {
	/// Represents the winner being on side `A`.
	A,
	/// Represents the winner being on side `B`.
	B,
}
impl std::convert::From<TournamentEdge> for TournamentRoundResult {
	fn from(e: TournamentEdge) -> Self {
		match e {
			TournamentEdge::A => Self::A,
			TournamentEdge::B => Self::B,
		}
	}
}
impl fmt::Display for TournamentRoundResult {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::A => write!(f, "A wins"),
			Self::B => write!(f, "B wins"),
		}
	}
}

/// Enum used for all errors in the crate.
#[derive(Debug, Clone, Copy)]
pub enum TournamentError {
	/// Returned when a [`Tournament`](struct.Tournament.html)'s internal [graph](struct.Tournament.html#method.graph) doesn't contain a certain [`NodeIndex`](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.NodeIndex.html)
	RoundNotFound(NodeIndex),
	/// Returned when a [`Tournament`](struct.Tournament.html) doesn't contain an entrant of a certain [`EntrantId`](struct.EntrantId.html)
	EntrantNotFound(EntrantId),
	/// Returned when a [`Tournament`](struct.Tournament.html)'s internal [graph](struct.Tournament.html#method.graph) is somehow malformed. This can be caused by manipulating the graph's structure after the tournament is instantiated.
	MalformedBracket,
	/// Returned when attempting to create a [`Tournament`](struct.Tournament.html) with zero entrants.
	NeedsAtLeastOneEntrant,
	/// Catchall other error.
	Other(&'static str),
	/// Returned by [`print_tournament`](fn.print_tournament.html) when some error prevents it from formatting the tree.
	PrintFailure,
}
