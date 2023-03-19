use std::{
	fmt,
	time::{Duration, Instant},
};

use crate::graph::{BlockId, Graph};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
	pub(crate) graph: Graph,
	pub(crate) compressed_graph: Graph,
	pub(crate) last: BlockId,
	pub(crate) updates: usize,
	pub(crate) rebuilds: usize,
	pub(crate) created_at: Instant,
	pub(crate) rebuilding_time: Duration,
}

impl Default for ControlFlowGraph {
	fn default() -> Self {
		Self::new()
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
			"\nNodes: {} ({})\nEdges: {} ({})\nConnections: Avg: {}, Max: {}\nUpdates: {}\nRebuilds: {}\nLifetime: {:?}\nTime spent rebuilding: {:?}",
			g.len(),
			self.graph.len(),
			g.nodes.values().map(|b| b.from.len()).sum::<usize>(),
			self.graph.nodes.values().map(|b| b.from.len()).sum::<usize>(),
			g.nodes.values().map(|b| b.from.len()).sum::<usize>() as f64 / g.len() as f64,
			g.nodes.values().map(|b| b.from.len()).max().unwrap(),
			self.updates,
			self.rebuilds,
			now - self.created_at,
			now2 - now,
		)
	}
}

impl ControlFlowGraph {
	pub fn new() -> Self {
		ControlFlowGraph {
			graph: Graph::default(),
			compressed_graph: Graph::default(),
			last: 0,
			updates: 0,
			rebuilds: 0,
			created_at: Instant::now(),
			rebuilding_time: Duration::new(0, 0),
		}
	}

	/// Insert a node connection. Returns true if the connection
	/// is new.
	pub fn update(&mut self, from: u64, to: u64) -> bool {
		let modified = self.graph.update(from, to);
		self.updates += 1;

		// Disabled for smoke test. When removing this, also reenable test1 below.
		return modified;

		// Only edit the compressed graph if this was a new link
		if modified {
			// If both links exist we can just add this one link
			if self.compressed_graph.nodes.contains_key(&to)
				&& self.compressed_graph.nodes.contains_key(&from)
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

#[cfg(test)]
mod test {
	use crate::control_flow::*;

	#[test]
	#[ignore] // Reenable along with removing the short-circuit in ControlFlowGraph::update
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
