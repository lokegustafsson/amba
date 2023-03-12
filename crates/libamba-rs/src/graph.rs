use std::{default::Default, mem};

// Aliased so we can swap them to binary versions easily.
type Set<T> = std::collections::HashSet<T>;
type Map<K, V> = std::collections::HashMap<K, V>;
type BlockId = u64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph(Map<u64, Block>);

impl Graph {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn compress(&mut self) {
		let m = &mut self.0;

		// Merge any nodes (x, y) where x always goes to y and
		// y always comes from x
		let mut to_merge = m
			.values()
			.filter(|l| l.from.len() == 1)
			.map(|l| (l, &m[l.from.iter().next().unwrap()]))
			.filter(|(_, r)| r.to.len() == 1)
			.map(|(l, r)| (l.id.min(r.id), l.id.max(r.id)))
			.collect::<Vec<_>>();

		// Sort pairs and make sure that a node is always
		// referred to by its merged name afterwards
		to_merge.sort_unstable_by(|x, y| x.cmp(y).reverse());
		for i in 1..to_merge.len() {
			let (done, to_do) = to_merge.split_at_mut(i);
			let (to, from) = done[i - 1];
			for (x, y) in to_do.iter_mut() {
				if *x == from {
					*x = to;
				}
				if *y == from {
					*y = to;
				}
			}
		}

		dbg!(&to_merge);

		// We always merge two nodes to the lowest one's id.
		// We can merge nodes highest first to make sure we
		// don't have any references that outlive the node.
		for (l, r) in to_merge.into_iter() {
			// Don't merge cycles
			if self.do_loop(l, r) {
				continue;
			}
			self.merge_nodes(l, r);
		}
	}

	fn do_loop(&self, l: u64, r: u64) -> bool {
		if l == r {
			return true;
		}
		let m = &self.0;
		m[&l].from.contains(&r) && m[&r].from.contains(&l)
	}

	fn merge_nodes(&mut self, l: u64, r: u64) {
		if l > r {
			self.merge_nodes(r, l);
			return;
		}
		if l == r {
			return;
		}

		assert!(self.0.contains_key(&l));
		assert!(self.0.contains_key(&r));

		let map = &mut self.0;

		// Take the union of both nodes' input and then remove the nodes themselves
		let to_r = mem::take(&mut map.get_mut(&r).unwrap().to);
		let from_r = mem::take(&mut map.get_mut(&r).unwrap().from);

		let l_ref = map.get_mut(&l).unwrap();

		for node in to_r.into_iter().filter(|&x| x != l && x != r) {
			l_ref.to.insert(node);
		}
		l_ref.to.remove(&l);
		l_ref.to.remove(&r);

		for node in from_r.into_iter().filter(|&x| x != l && x != r) {
			l_ref.from.insert(node);
		}
		l_ref.from.remove(&l);
		l_ref.from.remove(&r);

		// Remove the right node from the graph
		map.remove(&r);

		// And fix any pointers to the right node so that they point to the left node
		for node in map.values_mut() {
			if node.from.remove(&r) {
				node.from.insert(l);
			}
			if node.to.remove(&r) {
				node.to.insert(l);
			}
		}
	}

	/// Split `node` into two nodes, with the new node using the requested id if it's not already in use.
	/// Returns the id of the new node
	fn split_node(&mut self, node: u64, requested_id: u64) -> u64 {
		if self.0.contains_key(&requested_id) {
			return self.split_node(node, requested_id + 1);
		}
		todo!()
	}

	/// Verify that all node pairs have matching to and from
	#[cfg(test)]
	fn verify(&self) {
		let m = &self.0;

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlFlowGraph {
	graph: Graph,
	compressed_graph: Graph,
	last: BlockId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Block {
	id: BlockId,
	from: Set<BlockId>,
	to: Set<BlockId>,
}

impl<const N: usize, const M: usize> From<(BlockId, [BlockId; N], [BlockId; M])> for Block {
	fn from((id, f, t): (BlockId, [BlockId; N], [BlockId; M])) -> Self {
		Block {
			id,
			from: f.iter().cloned().collect(),
			to: t.iter().cloned().collect(),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::graph::*;

	/// 0 → 1 → 2
	#[test]
	fn straight_line() {
		let mut graph = Graph(
			[
				(0, (0, [], [1]).into()),
				(1, (1, [0], [2]).into()),
				(2, (2, [1], []).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph([(0, (0, [], []).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}

	/// 0 → 1
	#[test]
	fn short_line() {
		let mut graph = Graph(
			[(0, (0, [], [1]).into()), (1, (1, [0], []).into())]
				.into_iter()
				.collect(),
		);
		let expected = Graph([(0, (0, [], []).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}

	/// 2 → 1 → 0
	#[test]
	fn straight_line_rev() {
		let mut graph = Graph(
			[
				(0, (0, [1], []).into()),
				(1, (1, [2], [0]).into()),
				(2, (2, [], [1]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph([(0, (0, [], []).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		dbg!(&graph);
		graph.verify();
		assert_eq!(graph, expected);
	}

	/// 1 → 0
	#[test]
	fn short_line_rev() {
		let mut graph = Graph(
			[(0, (0, [1], []).into()), (1, (1, [], [0]).into())]
				.into_iter()
				.collect(),
		);
		let expected = Graph([(0, (0, [], []).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		dbg!(&graph);
		graph.verify();
		assert_eq!(graph, expected);
	}

	///   0
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   3
	#[test]
	fn diamond() {
		let mut graph = Graph(
			[
				(0, (0, [], [1, 2]).into()),
				(1, (1, [0], [3]).into()),
				(2, (2, [0], [3]).into()),
				(3, (3, [1, 2], []).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}

	///   3
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   0
	#[test]
	fn diamond_rev() {
		let mut graph = Graph(
			[
				(0, (0, [1, 2], []).into()),
				(1, (1, [3], [0]).into()),
				(2, (2, [3], [0]).into()),
				(3, (3, [], [1, 2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}

	/// 4 → 0
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 6   3
	#[test]
	fn diamond_on_stick() {
		let mut graph = Graph(
			[
				(0, (0, [4], [1, 2]).into()),
				(1, (1, [0], [3]).into()),
				(2, (2, [0], [3]).into()),
				(3, (3, [1, 2], []).into()),
				(4, (4, [5], [0]).into()),
				(5, (5, [6], [4]).into()),
				(6, (6, [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[
				(0, (0, [], [1, 2]).into()),
				(1, (1, [0], [3]).into()),
				(2, (2, [0], [3]).into()),
				(3, (3, [1, 2], []).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		dbg!(&graph);
		graph.verify();
		assert_eq!(graph, expected);
	}

	/// 6 → 3
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 4   0
	#[test]
	fn diamond_on_stick_rev() {
		let mut graph = Graph(
			[
				(0, (0, [1, 2], []).into()),
				(1, (1, [3], [0]).into()),
				(2, (2, [3], [0]).into()),
				(3, (3, [6], [1, 2]).into()),
				(4, (4, [], [5]).into()),
				(5, (5, [4], [6]).into()),
				(6, (6, [5], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[
				(0, (0, [1, 2], []).into()),
				(1, (1, [3], [0]).into()),
				(2, (2, [3], [0]).into()),
				(3, (3, [], [1, 2]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
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
		let mut graph = Graph(
			[
				(0, (0, [], [2]).into()),
				(1, (1, [], [2]).into()),
				(2, (2, [0, 1], [3]).into()),
				(3, (3, [2], [4, 5]).into()),
				(4, (4, [3], []).into()),
				(5, (5, [3], []).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[
				(0, (0, [], [2]).into()),
				(1, (1, [], [2]).into()),
				(2, (2, [0, 1], [4, 5]).into()),
				(4, (4, [2], []).into()),
				(5, (5, [2], []).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
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
		let mut graph = Graph(
			[
				(0, (0, [2], []).into()),
				(1, (1, [2], []).into()),
				(2, (2, [3], [0, 1]).into()),
				(3, (3, [4, 5], [2]).into()),
				(4, (4, [], [3]).into()),
				(5, (5, [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[
				(0, (0, [2], []).into()),
				(1, (1, [2], []).into()),
				(2, (2, [4, 5], [0, 1]).into()),
				(4, (4, [], [2]).into()),
				(5, (5, [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}

	/// 0   1
	///  ↘ ↙
	///   2
	///   ↓
	///   3
	///  ↙ ↘
	/// 4   5
	#[test]
	fn cross_split() {
		let mut graph = Graph(
			[
				(0, (0, [], [2]).into()),
				(1, (1, [], [2]).into()),
				(2, (2, [0, 1], [4, 5]).into()),
				(4, (4, [2], []).into()),
				(5, (5, [2], []).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[
				(0, (0, [], [2]).into()),
				(1, (1, [], [2]).into()),
				(2, (2, [0, 1], [3]).into()),
				(3, (3, [2], [4, 5]).into()),
				(4, (4, [3], []).into()),
				(5, (5, [3], []).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		let node = graph.split_node(2, 3);
		graph.verify();
		assert_eq!(graph, expected);
		assert_eq!(node, 3);
	}

	///   0
	///  ↙ ↖
	/// 1   3
	///  ↘ ↗
	///   2
	#[test]
	fn cycle() {
		let mut graph = Graph(
			[
				(0, (0, [3], [1]).into()),
				(1, (1, [0], [2]).into()),
				(2, (2, [1], [3]).into()),
				(3, (3, [2], [0]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[(0, (0, [1], [1]).into()), (1, (1, [0], [0]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}

	///   0
	///  ↙ ↖
	/// 3   1
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_rev() {
		let mut graph = Graph(
			[
				(0, (0, [1], [3]).into()),
				(1, (1, [2], [0]).into()),
				(2, (2, [3], [1]).into()),
				(3, (3, [0], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph(
			[(0, (0, [1], [1]).into()), (1, (1, [0], [0]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.verify();
		assert_eq!(graph, expected);
	}
}
