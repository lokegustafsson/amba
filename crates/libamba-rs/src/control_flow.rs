use std::{
	fmt,
	time::{Duration, Instant},
};

use crate::graph::{BlockId, Graph};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowGraph {
	pub(crate) graph: Graph,
	pub(crate) compressed_graph: Graph,
	pub(crate) last: BlockId,
	pub(crate) updates: usize,
	pub(crate) rebuilds: usize,
	pub(crate) spawned_at: Instant,
	pub(crate) rebuilding_time: Duration,
}

impl ControlFlowGraph {
	pub fn new() -> Self {
		ControlFlowGraph {
			graph: Graph::default(),
			compressed_graph: Graph::default(),
			last: 0,
			updates: 0,
			rebuilds: 0,
			spawned_at: std::time::Instant::now(),
			rebuilding_time: Duration::new(0, 0),
		}
	}

	/// Insert a node connection. Returns true if the connection
	/// is new.
	pub fn update(&mut self, from: u64, to: u64) -> bool {
		let modified = self.graph.update(from, to);
		self.updates += 1;
		return true;

		// Only edit the compressed graph if this was a new link
		if modified {
			// If both links exist we can just add this one link
			if self.compressed_graph.0.contains_key(&to)
				&& self.compressed_graph.0.contains_key(&from)
			{
				self.compressed_graph.update(from, to);
			} else {
				// but if either link is gone, we construct a new graph.
				// TODO: Figure out how to split nodes
				// and only compress new things.
				self.compressed_graph = self.graph.clone();
				self.rebuilds += 1;
			}
			let now = Instant::now();
			self.compressed_graph.compress();
			self.rebuilding_time += Instant::now() - now;
		}

		modified
	}
}

impl fmt::Display for ControlFlowGraph {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let now = Instant::now();
		let mut g = self.graph.clone();
		g.compress();
		let now2 = Instant::now();
		write!(
			f,
			"\nNodes: {} ({})\nUpdates: {}\nRebuilds: {}\nLifetime: {:?}\nTime spent rebuilding: {:?}",
			g.len(),
			self.graph.len(),
			self.updates,
			self.rebuilds,
			now - self.spawned_at,
			now2 - now,
		)
	}
}

#[cfg(test)]
mod test {
	use crate::control_flow::*;

	#[test]
	fn test1() {
		let mut cfg = ControlFlowGraph::new();
		// 0 → 1
		assert!(cfg.update(0, 1));
		assert_eq!(cfg.graph.len(), 2);
		assert_eq!(cfg.compressed_graph.len(), 1);
		assert!(!cfg.update(0, 1));

		// 0 → 1 → 2
		assert!(cfg.update(1, 2));
		assert_eq!(cfg.graph.len(), 3);
		assert_eq!(cfg.compressed_graph.len(), 1);
		assert!(!cfg.update(1, 2));

		// 0 → 1 → 2
		//     ↓
		//     3
		assert!(cfg.update(1, 3));
		assert_eq!(cfg.graph.len(), 4);
		assert_eq!(cfg.compressed_graph.len(), 3);
		assert!(!cfg.update(1, 3));

		// 0 → 1 → 2
		// ↑   ↓
		// 4   3
		assert!(cfg.update(4, 0));
		assert_eq!(cfg.graph.len(), 5);
		assert_eq!(cfg.compressed_graph.len(), 3);
		assert!(!cfg.update(4, 0));
	}
}
