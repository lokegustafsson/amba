use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::metadata::NodeMetadata;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphIpc {
	/// Metadata for nodes with implicit indices `0..metadata.len()`.
	pub metadata: Vec<NodeMetadata>,
	/// Directed edges between nodes, identified by their implicit indices
	pub edges: Vec<(usize, usize)>,
}

pub struct GraphIpcBuilder {
	graph: GraphIpc,
	existing_edges: HashSet<(u64, u64)>,
	id_to_idx: HashMap<u64, usize>,
}

impl Default for GraphIpcBuilder {
	fn default() -> Self {
		Self {
			graph: GraphIpc {
				metadata: Vec::new(),
				edges: Vec::new(),
			},
			existing_edges: HashSet::new(),
			id_to_idx: HashMap::new(),
		}
	}
}

impl GraphIpcBuilder {
	pub fn maybe_add_edge(&mut self, from: NodeMetadata, to: NodeMetadata) {
		let from_id = from.unique_id();
		let to_id = to.unique_id();
		if self.existing_edges.insert((from_id, to_id)) {
			let from = *self.id_to_idx.entry(from_id).or_insert_with(|| {
				self.graph.metadata.push(from);
				self.graph.metadata.len() - 1
			});
			let to = *self.id_to_idx.entry(to_id).or_insert_with(|| {
				self.graph.metadata.push(to);
				self.graph.metadata.len() - 1
			});
			self.graph.edges.push((from, to));
		}
	}

	pub fn get(&self) -> &GraphIpc {
		&self.graph
	}

	pub fn is_empty(&self) -> bool {
		self.id_to_idx.is_empty()
	}
}
