use crate::types::*;
#[doc(hidden)]
use petgraph::prelude::*;
#[doc(no_inline)]
use petgraph::{graph::NodeIndex, Graph};
use std::borrow::Cow;
use std::clone::Clone;
use std::default::Default;
use std::fmt;
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};

/// `Tournament<E, M, B>` is the core structure of the package. Creates a single-elimination tournament bracket.
/// - **`E`** - The entrant structs that will battle each other. Must implement `Debug`, `Display` and `Clone`.
/// 	- Internally, these are cloned, then stored as [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)`<`[`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html)`<E>>`, and are accessed through them after the tournament is created.
/// - **`M`** - The metadata struct that is added to rounds after being completed. Must implement `Debug`, `Display`, `Clone` and `Default`
/// - **`B`** - The battle system that solves rounds between two entrants of type `E`. Must implement [`BattleSystem<E, M>`](trait.BattleSystem.html)
#[derive(Debug)]
pub struct Tournament<
	E: Debug + Display + Clone,
	M: Debug + Display + Clone + Default,
	B: BattleSystem<E, M>,
> {
	graph: Graph<TournamentNode<M>, TournamentEdge>,
	entrants: Vec<Arc<RwLock<E>>>,
	grand_finals: NodeIndex,
	phantom: std::marker::PhantomData<B>,
	phantom_metadata: std::marker::PhantomData<M>,
}

impl<
		E: fmt::Debug + fmt::Display + Clone,
		M: Debug + Display + Clone + Default,
		B: BattleSystem<E, M>,
	> Tournament<E, M, B>
{
	/// Create a new `Tournament` from a `Vec<E>` of entrant structs. Brackets are assigned in the `Vec<E>`'s order.
	///
	/// # Example
	/// Create a `Tournament` that battles a vec of `u32`s
	/// ```
	/// use crate::{ MyBattleSystem, MyMetadata };
	///
	/// let entrants = vec![1, 2, 3, 23, 35, 483, 9494, 9, 0, 102, 48];
	///
	/// let t = Tournament::<u32, MyMetadata, MyBattleSystem>::new(entrants);
	/// ```
	pub fn new(entrants: Vec<E>) -> Result<Self> {
		if entrants.len() == 0 {
			return Err(TournamentError::NeedsAtLeastOneEntrant);
		}

		let entrant_arcs: Vec<Arc<RwLock<E>>> = entrants
			.into_iter()
			.map(|entrant| Arc::new(RwLock::new(entrant.clone())))
			.collect();

		let mut graph: Graph<TournamentNode<M>, TournamentEdge> = Graph::new();
		let mut entrant_ids: Vec<EntrantId> = vec![];
		for i in 0..entrant_arcs.len() {
			entrant_ids.push(EntrantId(i));
		}

		let grand_finals = if entrant_arcs.len() == 1 {
			graph.add_node(TournamentNode::Entrant(EntrantId(0)))
		} else {
			graph.add_node(TournamentNode::Round(TournamentRound::<M>::Incomplete))
		};

		graph = Self::add_layer(graph, grand_finals, entrant_ids);

		Ok(Tournament::<E, M, B> {
			graph,
			entrants: entrant_arcs,
			grand_finals,
			phantom: std::marker::PhantomData,
			phantom_metadata: std::marker::PhantomData,
		})
	}

	fn add_layer(
		old_graph: Graph<TournamentNode<M>, TournamentEdge>,
		parent: NodeIndex,
		entrants: Vec<EntrantId>,
	) -> Graph<TournamentNode<M>, TournamentEdge> {
		let mut graph = old_graph.clone();
		// println!("add_layer - P: {:?} (entrants: {})", parent, entrants.len());

		let incomplete = TournamentNode::<M>::Round(TournamentRound::Incomplete);

		// add bye + recursion on other 2
		if entrants.len() == 3 {
			let p = graph.add_node(incomplete.clone());
			let bye =
				graph.add_node(TournamentNode::Entrant(*entrants.get(0).unwrap()));
			graph.add_edge(parent, p, TournamentEdge::A);
			graph.add_edge(parent, bye, TournamentEdge::B);
			graph =
				Self::add_layer(graph, p, entrants.split_first().unwrap().1.to_vec());
		}
		// add regular
		else if entrants.len() == 2 {
			let a =
				graph.add_node(TournamentNode::Entrant(*entrants.get(0).unwrap()));
			let b =
				graph.add_node(TournamentNode::Entrant(*entrants.get(1).unwrap()));
			graph.add_edge(parent, a, TournamentEdge::A);
			graph.add_edge(parent, b, TournamentEdge::B);
		}
		// add nothing
		else if entrants.len() == 1 {
		}
		// do recursion
		else {
			let (slice_a, slice_b) = entrants.split_at(entrants.len() / 2);
			let vec_a = slice_a.to_vec();
			let vec_b = slice_b.to_vec();
			let p_a = graph.add_node(incomplete.clone());
			let p_b = graph.add_node(incomplete.clone());
			graph.add_edge(parent, p_a, TournamentEdge::A);
			graph.add_edge(parent, p_b, TournamentEdge::B);
			graph = Self::add_layer(graph, p_a, vec_a);
			graph = Self::add_layer(graph, p_b, vec_b);
		}

		graph
	}

	/// Created a new `Tournament` of a specified number of entrants, using a generation closure that returns a new entrant.
	///
	/// # Example
	/// Create a `Tournament` that battles 200 randomly generated `u32`s
	/// ```
	/// use rand::prelude::*;
	/// use crate::{ MyBattleSystem, MyMetadata };
	///
	/// let t = Tournament::<u32, MyMetadata, MyBattleSystem>::new_from_gen(
	/// 	200,
	/// 	|| random::<u32>()
	/// );
	/// ```
	pub fn new_from_gen(size: usize, gen: fn() -> E) -> Result<Self> {
		let mut entrants: Vec<E> = Vec::new();
		for _ in 0..size {
			entrants.push((gen)());
		}
		Self::new(entrants)
	}

	/// Get the number of entrants in the tournament.
	pub fn len_entrants(&self) -> usize {
		self.entrants.len()
	}

	/// Get the number of rounds in the tournament, complete and incomplete.
	pub fn len_rounds(&self) -> usize {
		let mut c = 0;
		for node in self.graph().node_indices() {
			match self.graph()[node] {
				TournamentNode::Entrant(_) => {}
				TournamentNode::Round(_) => c += 1,
			};
		}
		c
	}

	/// Get the number of completed rounds in the tournament.
	pub fn len_rounds_complete(&self) -> usize {
		let mut c = 0;
		for node in self.graph().node_indices() {
			if let TournamentNode::Round(TournamentRound::Complete {
				result: _,
				metadata: _,
			}) = self.graph()[node]
			{
				c += 1;
			}
		}
		c
	}

	/// Get the number of incomplete rounds in the tournament.
	pub fn len_rounds_incomplete(&self) -> usize {
		let mut c = 0;
		for node in self.graph().node_indices() {
			if let TournamentNode::Round(TournamentRound::Incomplete) =
				self.graph()[node]
			{
				c += 1;
			}
		}
		c
	}

	/// Get an `Arc<RwLock<E>>` encapsulating an entrant of specified [`EntrantId`](struct.EntrantId.html)
	pub fn entrant(&self, id: EntrantId) -> Arc<RwLock<E>> {
		self.entrants.get(id.0).unwrap().clone()
	}

	/// Get a ref to the [`NodeIndex`](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.NodeIndex.html) of the tournament's final round.
	pub fn grand_finals(&self) -> &NodeIndex {
		&self.grand_finals
	}

	/// Get a ref to the internal [`Graph`](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.Graph.html) used by the tournament. `ultra_tournament` is built using the [`petgraph`](https://docs.rs/petgraph/0.5.1/petgraph/index.html) crate.
	pub fn graph(&self) -> &Graph<TournamentNode<M>, TournamentEdge> {
		&self.graph
	}

	// ====================================
	fn _child_node(
		graph: &Graph<TournamentNode<M>, TournamentEdge>,
		id: NodeIndex,
		target: TournamentEdge,
	) -> Result<NodeIndex> {
		use TournamentError::*;
		let mut children = graph.edges_directed(id, petgraph::Direction::Outgoing);
		let child_edges = (
			children.next().ok_or(MalformedBracket)?,
			children.next().ok_or(MalformedBracket)?,
		);

		// TODO Why do these have to be backwards? But why? But why?
		if child_edges.0.weight() == &target {
			Ok(child_edges.1.target())
		} else if child_edges.1.weight() == &target {
			Ok(child_edges.0.target())
		} else {
			Err(MalformedBracket)
		}
	}

	/// Get the [`NodeIndex`](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.NodeIndex.html) of a round leading to one with the index `id`. Uses the [`TournamentEdge`](enum.TournamentEdge.html) to specify either [`A`](enum.TournamentEdge.html#variant.A) or [`B`](enum.TournamentEdge.html#variant.B)
	pub fn child_node(
		&self,
		id: NodeIndex,
		target: TournamentEdge,
	) -> Result<NodeIndex> {
		Self::_child_node(&self.graph, id, target)
	}
	fn _child_nodes(
		graph: &Graph<TournamentNode<M>, TournamentEdge>,
		id: NodeIndex,
	) -> Result<(NodeIndex, NodeIndex)> {
		Ok((
			Self::_child_node(graph, id, TournamentEdge::A)?,
			Self::_child_node(graph, id, TournamentEdge::B)?,
		))
	}

	/// Get a tuple of the [`NodeIndex`](https://docs.rs/petgraph/0.5.1/petgraph/graph/struct.NodeIndex.html)es of the previous rounds that lead to one with the index `id`, in the order `(A, B)`
	pub fn child_nodes(&self, id: NodeIndex) -> Result<(NodeIndex, NodeIndex)> {
		Self::_child_nodes(&self.graph, id)
	}

	fn _winner(
		graph: &Graph<TournamentNode<M>, TournamentEdge>,
		id: NodeIndex,
	) -> Result<Option<EntrantId>> {
		use TournamentError::*;
		use TournamentNode::*;

		let cur_res = graph.node_weight(id).ok_or(RoundNotFound(id))?;
		Ok(match cur_res {
			Entrant(entrant_id) => Some(*entrant_id),
			Round(round) => match round {
				TournamentRound::Incomplete => None,
				TournamentRound::<M>::Complete {
					result,
					metadata: _,
				} => match result {
					&TournamentRoundResult::A => Self::_winner(
						graph,
						Self::_child_node(graph, id, TournamentEdge::A)?,
					)?,
					&TournamentRoundResult::B => Self::_winner(
						graph,
						Self::_child_node(graph, id, TournamentEdge::B)?,
					)?,
				},
			},
		})
	}

	/// Get the [`EntrantId`](struct.EntrantId.html) of the solved winner of a particular round. Returns `None` if the round hasn't been calculated yet, or if the node is a [`TournamentNode::Entrant`](enum.TournamentNode.html#variant.Entrant) instead of a [`TournamentNode::Round`](enum.TournamentNode.html#variant.Round).
	pub fn winner(&self, id: NodeIndex) -> Result<Option<EntrantId>> {
		Self::_winner(&self.graph, id)
	}

	/// Identical to the [`winner()`](#method.winner) function, but returns the [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html)`<`[`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html)`<E>>` encapsulating the entrant instead of its [`EntrantId`](struct.EntrantId.html).
	pub fn winner_entrant(
		&self,
		id: NodeIndex,
	) -> Result<Option<Arc<RwLock<E>>>> {
		Ok(self.winner(id)?.map(|eid| self.entrant(eid)))
	}

	/// Solves all rounds in the tournament, as per [`solve_round()`](#method.solve_round), up to and including the returned by [`grand_finals()`](#method.grand_finals)
	pub fn solve(&mut self) -> Result<()> {
		self.solve_round(self.grand_finals)?;
		Ok(())
	}

	/// Solves rounds only up to the specified round.
	pub fn solve_round(
		&mut self,
		id: NodeIndex,
	) -> Result<TournamentRoundResult> {
		let mut graph = self.graph.clone();
		let res = self.solve_rec(&self.entrants.clone(), &mut graph, id)?;
		self.graph = graph;
		Ok(res)
	}
	fn solve_rec(
		&self,
		entrants: &Vec<Arc<RwLock<E>>>,
		old_graph: &mut Graph<TournamentNode<M>, TournamentEdge>,
		id: NodeIndex,
	) -> Result<TournamentRoundResult> {
		use TournamentError::*;
		use TournamentNode::*;
		let mut graph = old_graph.clone();
		let mut children =
			graph.neighbors_directed(id, petgraph::Direction::Outgoing);
		let a = children.next().ok_or(Other("Child A not found"))?;
		let b = children.next().ok_or(Other("Child B not found"))?;

		macro_rules! do_bye {
			($ent_bye:expr, $other_node:expr, $bye_is:expr) => {{
				let ent_round = Self::_winner(&graph, $other_node)?.unwrap_or({
					self.solve_rec(entrants, &mut graph, $other_node)?;
					Self::_winner(&graph, $other_node)?
						.ok_or(Other("Solving Bye failed"))?
				});
				let arc_bye = entrants
					.get($ent_bye.0)
					.ok_or(EntrantNotFound($ent_bye))?
					.clone();
				let arc_round = entrants
					.get(ent_round.0)
					.ok_or(EntrantNotFound(ent_round))?
					.clone();
				match $bye_is {
					TournamentEdge::A => (
						arc_bye.clone(),
						arc_round.clone(),
						B::battle(arc_bye.clone(), arc_round.clone()),
					),
					TournamentEdge::B => (
						arc_round.clone(),
						arc_bye.clone(),
						B::battle(arc_round.clone(), arc_bye.clone()),
					),
				}
				}};
		}

		let (arc_a, arc_b, res) = match (
			graph.node_weight(a).unwrap().clone(),
			graph.node_weight(b).unwrap().clone(),
		) {
			(Entrant(id_a), Entrant(id_b)) => {
				let arc_a = entrants.get(id_a.0).ok_or(EntrantNotFound(id_a))?.clone();
				let arc_b = entrants.get(id_b.0).ok_or(EntrantNotFound(id_b))?.clone();
				(
					arc_a.clone(),
					arc_b.clone(),
					B::battle(arc_a.clone(), arc_b.clone()),
				)
			}
			(Entrant(ent_bye), Round(_)) => do_bye!(ent_bye, b, TournamentEdge::A),
			(Round(_), Entrant(ent_bye)) => do_bye!(ent_bye, a, TournamentEdge::B),
			(Round(_), Round(_)) => {
				let ent_a = Self::_winner(&graph, a)?.unwrap_or({
					self.solve_rec(entrants, &mut graph, a)?;
					Self::_winner(&graph, a)?
						.ok_or(Other("Finding winner failed for A"))?
				});
				let ent_b = Self::_winner(&graph, b)?.unwrap_or({
					self.solve_rec(entrants, &mut graph, b)?;
					Self::_winner(&graph, b)?
						.ok_or(Other("Finding winner failed for B"))?
				});
				let arc_a =
					entrants.get(ent_a.0).ok_or(EntrantNotFound(ent_a))?.clone();
				let arc_b =
					entrants.get(ent_b.0).ok_or(EntrantNotFound(ent_b))?.clone();
				(
					arc_a.clone(),
					arc_b.clone(),
					B::battle(arc_a.clone(), arc_b.clone()),
				)
			}
		};

		let (result, metadata) = match res {
			BattleResult::Solved(round_result, metadata) => (round_result, metadata),
			BattleResult::Tie => B::tiebreaker(arc_a, arc_b),
		};
		let weight = graph.node_weight_mut(id).ok_or(RoundNotFound(id))?;
		*weight = TournamentNode::Round(TournamentRound::<M>::Complete {
			result,
			metadata,
		});

		*old_graph = graph;
		Ok(result)
	}
}

impl<
		E: fmt::Debug + fmt::Display + Clone,
		M: Debug + Display + Clone + Default,
		B: BattleSystem<E, M>,
	> fmt::Display for Tournament<E, M, B>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "TODO")
	}
}

#[derive(Clone)]
struct PrintTournament<
	'a,
	E: fmt::Debug + fmt::Display + Clone,
	M: Debug + Display + Clone + Default,
	B: BattleSystem<E, M>,
>(&'a Tournament<E, M, B>, NodeIndex);

impl<'a, E, M, B> ptree::TreeItem for PrintTournament<'a, E, M, B>
where
	E: fmt::Debug + fmt::Display + Clone,
	M: Debug + Display + Clone + Default,
	B: BattleSystem<E, M>,
{
	type Child = Self;
	fn write_self<W: std::io::Write>(
		&self,
		f: &mut W,
		style: &ptree::Style,
	) -> std::io::Result<()> {
		if let Some(eid) = self.0.winner(self.1).unwrap() {
			let e_arc = self.0.entrant(eid);
			let e_value = e_arc.read().unwrap();
			match self.0.graph.node_weight(self.1).unwrap() {
				TournamentNode::Entrant(_) => write!(f, "{}", style.paint(e_value)),
				TournamentNode::Round(round) => write!(
					f,
					"{}",
					format!("{} ({})", style.paint(e_value), style.paint(round))
				),
			}
		} else {
			write!(f, "{}", style.paint("Incomplete"))
		}
	}
	fn children(&self) -> Cow<[Self::Child]> {
		let v: Vec<_> = self
			.0
			.graph
			.neighbors_directed(self.1, Direction::Outgoing)
			.map(|i| PrintTournament(self.0, i))
			.collect();
		Cow::from(v)
	}
}

/// Pretty-print a tournament using the crate [`ptree`](https://docs.rs/ptree/0.2.1/ptree/)
pub fn print_tournament<
	E: fmt::Debug + fmt::Display + Clone,
	M: Debug + Display + Clone + Default,
	B: BattleSystem<E, M> + Clone,
>(
	t: &Tournament<E, M, B>,
) -> Result<()> {
	#[doc(hidden)]
	use ptree::print_tree;
	print_tree(&PrintTournament(t, t.grand_finals))
		.or(Err(TournamentError::PrintFailure))
}
