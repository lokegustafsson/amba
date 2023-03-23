use std::{collections::BTreeSet, default::Default, mem};

use crate::small_set::SmallU64Set;

// Aliased so we can swap them to BTree versions easily.
pub(crate) type Set<T> = std::collections::BTreeSet<T>;
pub(crate) type Map<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Debug, Clone, Default)]
pub struct Block {
	pub(crate) id: u64,
	pub(crate) from: SmallU64Set,
	pub(crate) to: SmallU64Set,
	pub(crate) of: SmallU64Set,
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
	pub(crate) nodes: Map<u64, Block>,
	pub(crate) merges: Map<u64, u64>,
}

impl Graph {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn with_nodes(nodes: Map<u64, Block>) -> Self {
		Graph {
			nodes,
			merges: Map::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	pub fn get(&mut self, mut idx: u64) -> Option<&Block> {
		idx = translate(idx, &mut self.merges);
		self.nodes.get(&idx)
	}

	pub fn get_mut(&mut self, mut idx: u64) -> Option<&mut Block> {
		idx = translate(idx, &mut self.merges);
		self.nodes.get_mut(&idx)
	}

	pub fn apply_merges(&mut self) {
		for Block { from, to, .. } in self.nodes.values_mut() {
			*from = from
				.iter()
				.copied()
				.map(|x| translate(x, &mut self.merges))
				.collect();
			*to = to
				.iter()
				.copied()
				.map(|x| translate(x, &mut self.merges))
				.collect();
		}
		self.merges = Map::new();
	}

	/// Insert a node connection. Returns true if the connection
	/// is new.
	pub fn update(&mut self, from: u64, to: u64) -> bool {
		let mut modified = false;
		self.nodes
			.entry(from)
			.and_modify(|node| {
				modified |= node.to.insert(to);
			})
			.or_insert_with(|| {
				modified = true;
				let to = [to].into_iter().collect::<SmallU64Set>();
				let of = [from].into_iter().collect::<SmallU64Set>();
				Block {
					id: from,
					to,
					from: Default::default(),
					of,
				}
			});
		self.nodes
			.entry(to)
			.and_modify(|node| {
				modified |= node.from.insert(from);
			})
			.or_insert_with(|| {
				modified = true;
				let from = [from].into_iter().collect::<SmallU64Set>();
				let of = [to].into_iter().collect::<SmallU64Set>();
				Block {
					id: to,
					to: Default::default(),
					from,
					of,
				}
			});

		modified
	}

	/// Revert compression of nodes and then update their connections.
	/// Returns the reverted nodes
	pub fn revert_and_update(&mut self, source: &Graph, from: u64, to: u64) -> SmallU64Set {
		let mut nodes = SmallU64Set::new();

		// `self` and `source` *should* contain all the same
		// keys except in the case where this is the operation
		// where we're adding the keys to `self`
		if source.nodes.contains_key(&from) {
			if let Some(block) = self.get(from) {
				for node in block.of.iter().copied() {
					nodes.insert(node);
				}
			}
		}
		if source.nodes.contains_key(&to) {
			if let Some(block) = self.get(to) {
				for node in block.of.iter().copied() {
					nodes.insert(node);
				}
			}
		}

		for node in nodes.iter() {
			self.merges.remove(node);
			let value = source.nodes.get(node).unwrap().clone();
			self.nodes.insert(*node, value);
		}

		self.update(from, to);

		nodes.insert(from);
		nodes.insert(to);
		nodes
	}

	/// Compresses graph by merging every node pair that always go
	/// from one to the other
	pub fn compress(&mut self) {
		let m = &mut self.nodes;

		// Visit every node in arbitrary order.
		// We have to check (a, b) AND (b, a) seperately
		//  because we have a directed *cyclic* graph.
		// Following a depth-first order would just require
		//  a visited collection for no benefit.
		// We have to traverse the graph twice anyway because
		//  of the borrow checker.
		// The first value is always the smallest and the
		//  merged node will take the id of the smallest of
		//  the parents.
		let to_merge = m
			.values()
			.filter(|l| l.from.len() == 1)
			.map(|l| {
				let key = translate(l.from.get_any(), &mut self.merges);
				(l, &m[&key])
			})
			.filter(|(_, r)| r.to.len() == 1)
			.map(|(l, r)| (l.id, r.id))
			.collect::<Set<_>>();

		for (mut l, mut r) in to_merge.into_iter() {
			l = translate(l, &mut self.merges);
			r = translate(r, &mut self.merges);

			let x = l.min(r);
			let y = l.max(r);
			self.merge_nodes(x, y);
		}
	}

	/// Compress around given candidates. If a candidate gets
	/// compressed its neighbours will be checked too, growing out
	/// from there.
	pub fn compress_with_hint(&mut self, nodes: SmallU64Set) {
		fn inner(graph: &mut Graph, mut queued: BTreeSet<(u64, u64)>) {
			while let Some((mut from, mut to)) = queued.pop_first() {
				from = translate(from, &mut graph.merges);
				to = translate(to, &mut graph.merges);
				(from, to) = (from.min(to), from.max(to));

				if !graph.are_mergable_link(from, to) {
					continue;
				}
				let this = graph.merge_nodes(from, to);
				let node = &graph.nodes[&this];

				let tos = node.to.iter().filter(|&&n| n != this);
				let froms = node.from.iter().filter(|&&n| n != this);

				for &connection in tos.chain(froms) {
					queued.insert((connection, this));
				}
			}
		}

		let queued = nodes
			.into_iter()
			.flat_map(|n| {
				let node = self.nodes.get(&n).unwrap();
				let tos = node.to.iter().copied().map(move |t| (n, t));
				let froms = node.from.iter().copied().map(move |f| (f, n));

				tos.chain(froms)
			})
			.collect();

		inner(self, queued);
	}

	fn are_mergable_link(&mut self, mut l: u64, mut r: u64) -> bool {
		l = translate(l, &mut self.merges);
		r = translate(r, &mut self.merges);

		let mut f = |x, y| {
			if self.nodes[&x].to.len() != 1 || self.nodes[&y].from.len() != 1 {
				return false;
			}

			let to_link = self.nodes[&x].to.get_any();
			let from_link = self.nodes[&y].from.get_any();

			translate(to_link, &mut self.merges) == y && translate(from_link, &mut self.merges) == x
		};

		f(l, r) || f(r, l)
	}

	fn are_loop(&mut self, l: u64, r: u64) -> bool {
		if l == r {
			return true;
		}
		let m = &self.nodes;

		let there = m[&l]
			.from
			.iter()
			.map(|&i| translate(i, &mut self.merges))
			.any(|x| x == r);
		let back_again = m[&r]
			.from
			.iter()
			.map(|&i| translate(i, &mut self.merges))
			.any(|x| x == l);

		there && back_again
	}

	/// Returns the id of the merged node
	pub fn merge_nodes(&mut self, l: u64, r: u64) -> u64 {
		if l > r {
			return self.merge_nodes(r, l);
		}
		if l == r {
			return l;
		}

		assert!(self.nodes.contains_key(&l));
		assert!(self.nodes.contains_key(&r));

		let are_loop = self.are_loop(l, r);
		let map = &mut self.nodes;

		// Must be after are_loop
		self.merges.insert(r, l);
		self.merges.remove(&l);

		let combine_sets = |mut left_set: SmallU64Set, mut right_set: SmallU64Set| -> SmallU64Set {
			if left_set.len() < right_set.len() {
				mem::swap(&mut left_set, &mut right_set);
			}
			for node in right_set.into_iter() {
				left_set.insert(node);
			}

			left_set
		};

		// Take the union of both nodes' input and then remove
		// the nodes themselves
		let r_ = map.get_mut(&r).unwrap();
		let to_r = mem::take(&mut r_.to);
		let from_r = mem::take(&mut r_.from);
		let of_r = mem::take(&mut r_.of);

		let l_ = map.get_mut(&l).unwrap();
		let to_l = mem::take(&mut l_.to);
		let from_l = mem::take(&mut l_.from);
		let of_l = mem::take(&mut l_.of);

		l_.to = combine_sets(to_l, to_r);
		l_.from = combine_sets(from_l, from_r);
		l_.of = combine_sets(of_l, of_r);

		for parent in l_.of.iter() {
			l_.to.remove(parent);
			l_.from.remove(parent);
		}

		// Restore loop if they were a loop beforehand
		if are_loop {
			l_.from.insert(l);
			l_.to.insert(l);
		}
		l_.of.insert(l);
		l_.of.insert(r);

		// Remove the right node from the graph
		map.remove(&r);

		l
	}

	/// Verify that all node pairs have matching to and from
	#[cfg(test)]
	fn verify(&self) {
		let m = &self.nodes;

		for (k, v) in m.iter() {
			for out in v.from.iter() {
				assert!(
					m[out].to.contains(k),
					"{out}.to contains {k}?\n{self:#?}"
				);
			}
		}

		for (k, v) in m.iter() {
			for to in v.to.iter() {
				assert!(
					m[to].from.contains(k),
					"{to}.from contains {k}?\n{self:#?}"
				);
			}
		}
	}
}

// Seems linear, but in practice, unless the entire
// world is one long straight line ends up never
// taking more than a single step.
fn translate(key: u64, map: &mut Map<u64, u64>) -> u64 {
	match map.get(&key) {
		Some(k) => {
			let res = translate(*k, map);
			*map.get_mut(&key).unwrap() = res;
			res
		}
		None => key,
	}
}

impl<const N: usize, const M: usize, const O: usize> From<(u64, [u64; N], [u64; M], [u64; O])>
	for Block
{
	fn from((id, f, t, o): (u64, [u64; N], [u64; M], [u64; O])) -> Self {
		Block {
			id,
			from: f.into_iter().collect(),
			to: t.into_iter().collect(),
			of: o.into_iter().collect(),
		}
	}
}

#[cfg(test)]
mod test {
	use proptest::{
		prelude::*,
		test_runner::{Config, TestRunner},
	};

	use crate::graph::*;

	impl PartialEq for Block {
		fn eq(&self, other: &Self) -> bool {
			// Deconstructed so that it will cause a
			// compilation error if we add a field and
			// forget to update this
			let Block {
				id: l_id,
				from: l_from,
				to: l_to,
				of: l_of,
			} = self;
			let Block {
				id: r_id,
				from: r_from,
				to: r_to,
				of: r_of,
			} = other;

			l_id == r_id && l_from == r_from && l_to == r_to && l_of == r_of
		}
	}

	impl Eq for Block {}

	fn compare_behaviour(instructions: Vec<(u64, u64)>) -> Result<(), TestCaseError> {
		let mut fast = Graph::new();
		let mut slow = Graph::new();
		for (from, to) in instructions.into_iter() {
			slow.update(from, to);
			let reverted = fast.revert_and_update(&slow, from, to);
			fast.compress_with_hint(reverted);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(&fast_.nodes, &clone.nodes);
		}

		Ok(())
	}

	fn generator(max_id: u64, instruction_count: usize) -> impl Strategy<Value = Vec<(u64, u64)>> {
		let node_pair = (0..max_id, 0..max_id);
		prop::collection::vec(node_pair, instruction_count)
	}

	#[test]
	fn compare_10_20() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(10, 20), compare_behaviour).unwrap();
	}

	/// 0 → 1 → 2
	#[test]
	fn straight_line() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 2 → 1 → 0
	#[test]
	fn straight_line_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [], [1], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0 → 1
	#[test]
	fn short_line() {
		#[rustfmt::skip]
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [], [1]).into())
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes([(0, (0, [], [], [0, 1]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 1 → 0
	#[test]
	fn short_line_rev() {
		let mut graph = Graph::with_nodes(
			[(0, (0, [1], [], [0]).into()), (1, (1, [], [0], [1]).into())]
				.into_iter()
				.collect(),
		);
		let expected = Graph::with_nodes([(0, (0, [], [], [0, 1]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   3
	#[test]
	fn diamond() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   3
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   0
	#[test]
	fn diamond_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4 → 0
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 6   3
	#[test]
	fn diamond_on_stick() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [4], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
				(4, (4, [5], [0], [4]).into()),
				(5, (5, [6], [4], [5]).into()),
				(6, (6, [], [5], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [0, 4, 5, 6]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 6 → 3
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 4   0
	#[test]
	fn diamond_on_stick_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [6], [1, 2], [3]).into()),
				(4, (4, [], [5], [4]).into()),
				(5, (5, [4], [6], [5]).into()),
				(6, (6, [5], [3], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [3, 4, 5, 6]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	///  ↘ ↙
	///   2
	///   ↓
	///   3
	///  ↙ ↘
	/// 4   5
	#[test]
	fn cross() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [3], [2]).into()),
				(3, (3, [2], [4, 5], [3]).into()),
				(4, (4, [3], [], [4]).into()),
				(5, (5, [3], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [4, 5], [2, 3]).into()),
				(4, (4, [2], [], [4]).into()),
				(5, (5, [2], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4   5
	///  ↘ ↙
	///   3
	///   ↓
	///   2
	///  ↙ ↘
	/// 0   1
	#[test]
	fn cross_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [3], [0, 1], [2]).into()),
				(3, (3, [4, 5], [2], [3]).into()),
				(4, (4, [], [3], [4]).into()),
				(5, (5, [], [3], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [4, 5], [0, 1], [2, 3]).into()),
				(4, (4, [], [2], [4]).into()),
				(5, (5, [], [2], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 1   3
	///  ↘ ↗
	///   2
	#[test]
	fn cycle() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [3], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [3], [2]).into()),
				(3, (3, [2], [0], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 1, 2, 3]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 3   1
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [3], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [3], [1], [2]).into()),
				(3, (3, [0], [2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 1, 2, 3]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	/// ↓   ↓
	/// 2   3
	///  ↘ ↙
	///   4
	#[test]
	fn v() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [3], [1]).into()),
				(2, (2, [0], [4], [2]).into()),
				(3, (3, [1], [4], [3]).into()),
				(4, (4, [2, 3], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [4], [0, 2]).into()),
				(1, (1, [], [4], [1, 3]).into()),
				(4, (4, [0, 1], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0 → 1 → 2
	#[test]
	fn straight_line_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 2 → 1 → 0
	#[test]
	fn straight_line_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [], [1], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   3
	#[test]
	fn diamond_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   3
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   0
	#[test]
	fn diamond_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress_with_hint(3, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4 → 0
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 6   3
	#[test]
	fn diamond_on_stick_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [4], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
				(4, (4, [5], [0], [4]).into()),
				(5, (5, [6], [4], [5]).into()),
				(6, (6, [], [5], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [0, 4, 5, 6]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(5, 4);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 6 → 3
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 4   0
	#[test]
	fn diamond_on_stick_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [6], [1, 2], [3]).into()),
				(4, (4, [], [5], [4]).into()),
				(5, (5, [4], [6], [5]).into()),
				(6, (6, [5], [3], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [3, 4, 5, 6]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(5, 6);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	///  ↘ ↙
	///   2
	///   ↓
	///   3
	///  ↙ ↘
	/// 4   5
	#[test]
	fn cross_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [3], [2]).into()),
				(3, (3, [2], [4, 5], [3]).into()),
				(4, (4, [3], [], [4]).into()),
				(5, (5, [3], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [4, 5], [2, 3]).into()),
				(4, (4, [2], [], [4]).into()),
				(5, (5, [2], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(2, 3);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4   5
	///  ↘ ↙
	///   3
	///   ↓
	///   2
	///  ↙ ↘
	/// 0   1
	#[test]
	fn cross_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [3], [0, 1], [2]).into()),
				(3, (3, [4, 5], [2], [3]).into()),
				(4, (4, [], [3], [4]).into()),
				(5, (5, [], [3], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [4, 5], [0, 1], [2, 3]).into()),
				(4, (4, [], [2], [4]).into()),
				(5, (5, [], [2], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(3, 2);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 1   3
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [3], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [3], [2]).into()),
				(3, (3, [2], [0], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 1, 2, 3]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 3   1
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [3], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [3], [1], [2]).into()),
				(3, (3, [0], [2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 1, 2, 3]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(2, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	/// ↓   ↓
	/// 2   3
	///  ↘ ↙
	///   4
	#[test]
	fn v_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [3], [1]).into()),
				(2, (2, [0], [4], [2]).into()),
				(3, (3, [1], [4], [3]).into()),
				(4, (4, [2, 3], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [4], [0, 2]).into()),
				(1, (1, [], [3], [1]).into()),
				(3, (3, [1], [4], [3]).into()),
				(4, (4, [0, 3], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(2, 4);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0 → 1 → 2
	/// ↓
	/// 3
	#[test]
	fn incremental_l() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected_1 =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		let expected_2 = Graph::with_nodes(
			[
				(0, (0, [], [1, 3], [0]).into()),
				(1, (1, [0], [], [1, 2]).into()),
				(3, (3, [0], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected_1.verify();
		expected_2.verify();

		let raw = graph.clone();
		graph.compress();
		graph.apply_merges();
		assert_eq!(&graph.nodes, &expected_1.nodes);
		let revert = graph.revert_and_update(&raw, 0, 3);
		graph.compress_with_hint(revert);
		graph.apply_merges();
		assert_eq!(&graph.nodes, &expected_2.nodes);
	}

	#[test]
	fn incremental_generated_1() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			let reverted = fast.revert_and_update(&slow, from, to);
			fast.compress_with_hint(reverted);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(9, 8);
		cycle(0, 9);
		cycle(1, 8);
	}
}
