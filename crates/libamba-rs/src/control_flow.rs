use crate::graph::{BlockId, Graph};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlFlowGraph {
	graph: Graph,
	compressed_graph: Graph,
	last: BlockId,
}

impl ControlFlowGraph {
	pub fn new() -> Self {
		Default::default()
	}

	/// Insert a node connection. Returns true if the connection
	/// is new.
	pub fn update(&mut self, from: u64, to: u64) -> bool {
		let modified = self.graph.update(from, to);

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
			}
			self.compressed_graph.compress();
		}

		modified
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
