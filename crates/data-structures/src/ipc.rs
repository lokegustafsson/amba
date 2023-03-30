use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeMetadata {
	id: u64,
}

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

impl GraphIpcBuilder {
	pub fn new() -> Self {
		Self {
			graph: GraphIpc {
				metadata: Vec::new(),
				edges: Vec::new(),
			},
			existing_edges: HashSet::new(),
			id_to_idx: HashMap::new(),
		}
	}

	pub fn maybe_add_edge(&mut self, id_from: u64, id_to: u64) {
		if self.existing_edges.insert((id_from, id_to)) {
			let from = *self.id_to_idx.entry(id_from).or_insert_with(|| {
				self.graph.metadata.push(NodeMetadata { id: id_from });
				self.graph.metadata.len() - 1
			});
			let to = *self.id_to_idx.entry(id_to).or_insert_with(|| {
				self.graph.metadata.push(NodeMetadata { id: id_to });
				self.graph.metadata.len() - 1
			});
			self.graph.edges.push((from, to));
		}
	}

	pub fn get(&self) -> &GraphIpc {
		&self.graph
	}
}
