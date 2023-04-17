use std::{
	collections::BTreeMap,
	fmt,
	time::{Duration, Instant},
};

use ipc::NodeMetadata;

use crate::Graph;

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
	pub graph: Graph,
	pub(crate) compressed_graph: Graph,
	pub(crate) updates: usize,
	pub(crate) rebuilds: usize,
	pub(crate) created_at: Instant,
	pub(crate) rebuilding_time: Duration,
	pub(crate) metadata: Vec<NodeMetadata>,
	pub(crate) meta_mapping_unique_id_to_index: BTreeMap<u64, usize>,
}

impl From<&ControlFlowGraph> for ipc::GraphIpc {
	fn from(value: &ControlFlowGraph) -> Self {
		todo!()
	}
}

impl Default for ControlFlowGraph {
	fn default() -> Self {
		Self::new()
	}
}

impl fmt::Display for ControlFlowGraph {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let now = Instant::now();
		write!(
			f,
			concat!(
				"Nodes: {} ({})\n",
				"Edges: {} ({})\n",
				"Connections: Avg from: {}, Avg to: {}, Max: {}\n",
				"Updates: {}\n",
				"Rebuilds: {}\n",
				"Lifetime: {:?}\n",
				"Time spent rebuilding: {:?}"
			),
			self.compressed_graph.len(),
			self.graph.len(),
			self.compressed_graph
				.nodes
				.values()
				.map(|b| b.from.len())
				.sum::<usize>(),
			self.graph
				.nodes
				.values()
				.map(|b| b.from.len())
				.sum::<usize>(),
			self.compressed_graph
				.nodes
				.values()
				.map(|b| b.from.len())
				.sum::<usize>() as f64
				/ self.compressed_graph.len() as f64,
			self.compressed_graph
				.nodes
				.values()
				.map(|b| b.to.len())
				.sum::<usize>() as f64
				/ self.compressed_graph.len() as f64,
			self.compressed_graph
				.nodes
				.values()
				.map(|b| b.from.len().max(b.to.len()))
				.max()
				.unwrap_or_default(),
			self.updates,
			self.rebuilds,
			now - self.created_at,
			self.rebuilding_time,
		)
	}
}

impl ControlFlowGraph {
	pub fn new() -> Self {
		ControlFlowGraph {
			graph: Graph::default(),
			compressed_graph: Graph::default(),
			updates: 0,
			rebuilds: 0,
			created_at: Instant::now(),
			rebuilding_time: Duration::new(0, 0),
			metadata: Vec::new(),
			meta_mapping_unique_id_to_index: BTreeMap::new(),
		}
	}

	/// Insert a node connection. Returns true if the connection
	/// is new.
	pub fn update(&mut self, from_meta: NodeMetadata, to_meta: NodeMetadata) -> bool {
		let from = self.update_metadata(from_meta);
		let to = self.update_metadata(to_meta);

		let now = Instant::now();
		let modified = self.graph.update(from, to);
		self.updates += 1;

		// Only edit the compressed graph if this was a new link
		if modified {
			let reverted = self
				.compressed_graph
				.revert_and_update(&self.graph, from, to);

			self.rebuilds += 1;
			self.compressed_graph.compress_with_hint(reverted);
		}

		self.rebuilding_time += Instant::now() - now;
		modified
	}

	fn update_metadata(&mut self, node: NodeMetadata) -> u64 {
		let id = node.unique_id();
		let index: usize = *self
			.meta_mapping_unique_id_to_index
			.entry(id)
			.or_insert_with(|| {
				let seq_index = self.metadata.len();
				self.metadata.push(node);
				seq_index
			});
		index as u64
	}
}

#[cfg(test)]
mod test {
	use crate::control_flow::*;

	fn node(i: u32) -> NodeMetadata {
		NodeMetadata {
			symbolic_state_id: i,
			basic_block_vaddr: None,
			basic_block_generation: None,
		}
	}

	#[test]
	fn test1() {
		let mut cfg = ControlFlowGraph::new();
		// 0 → 1
		assert!(cfg.update(node(0), node(1)));
		assert_eq!(cfg.graph.len(), 2);
		assert_eq!(cfg.compressed_graph.len(), 1);
		assert!(!cfg.update(node(0), node(1)));

		// 0 → 1 → 2
		assert!(cfg.update(node(1), node(2)));
		assert_eq!(cfg.graph.len(), 3);
		assert_eq!(cfg.compressed_graph.len(), 1);
		assert!(!cfg.update(node(1), node(2)));

		// 0 → 1 → 2
		//     ↓
		//     3
		assert!(cfg.update(node(1), node(3)));
		assert_eq!(cfg.graph.len(), 4);
		assert_eq!(cfg.compressed_graph.len(), 3);
		assert!(!cfg.update(node(1), node(3)));

		// 0 → 1 → 2
		// ↑   ↓
		// 4   3
		assert!(cfg.update(node(4), node(0)));
		assert_eq!(cfg.graph.len(), 5);
		assert_eq!(cfg.compressed_graph.len(), 3);
		assert!(!cfg.update(node(4), node(0)));
	}
}
