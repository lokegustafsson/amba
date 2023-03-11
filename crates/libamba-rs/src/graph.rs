use std::default::Default;

type Set<T> = std::collections::HashSet<T>;
type Map<K, V> = std::collections::HashMap<K, V>;
type BlockId = u64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Graph(Map<u64, Block>);

impl Graph {
	fn compress(&mut self) {}

	/// Verify that all node pairs have matching to and from
	#[cfg(test)]
	fn verify(&self) {
		let m = &self.0;

		for (k, v) in m.iter() {
			for out in v.from.iter() {
				assert!(m[out].to.contains(k));
			}
		}

		for (k, v) in m.iter() {
			for to in v.to.iter() {
				assert!(m[to].from.contains(k));
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

	/// 0 -> 1 -> 2
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

	///   0
	///  / \
	/// 1   2
	///  \ /
	///   3
	#[test]
	fn diamond() {
		let mut graph = Graph(
			[
				(0, (0, [], [1, 2]).into()),
				(1, (1, [0], [3]).into()),
				(2, (2, [0], [3]).into()),
				(3, (2, [1, 2], []).into()),
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

	/// 5 -> 4 -> 0
	/// ^        / \
	/// 6       1   2
	///          \ /
	///           3
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
				(3, (2, [1, 2], []).into()),
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
}
