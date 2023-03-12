use crate::graph::{Block, BlockId, Graph};

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
		let mut modified = false;
		self.graph
			.0
			.entry(from)
			.and_modify(|node| {
				modified |= node.to.insert(to);
			})
			.or_insert_with(|| {
				modified = true;
				Block {
					id: from,
					to: [to].into_iter().collect(),
					from: Default::default(),
				}
			});
		self.graph
			.0
			.entry(to)
			.and_modify(|node| {
				modified |= node.from.insert(from);
			})
			.or_insert_with(|| {
				modified = true;
				Block {
					id: to,
					to: Default::default(),
					from: [from].into_iter().collect(),
				}
			});

		if modified {
			// Do cleverly when performance actually needs it
			self.compressed_graph = self.graph.clone();
			self.compressed_graph.compress();
		}

		modified
	}
}

}
