use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{Graph, Node, SmallU64Set};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeMetadata {
	id: u64,
	of: SmallU64Set,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphIpc {
	/// Metadata for nodes with implicit indices `0..metadata.len()`.
	pub metadata: Vec<NodeMetadata>,
	/// Directed edges between nodes, identified by their implicit indices
	pub edges: Vec<(usize, usize)>,
}

impl From<&GraphIpc> for Graph {
	fn from(ipc: &GraphIpc) -> Graph {
		let mut merges = BTreeMap::new();
		let mut nodes: BTreeMap<u64, Node> = ipc
			.metadata
			.iter()
			.map(|NodeMetadata { id, of }| {
				let node = Node {
					id: *id,
					from: SmallU64Set::new(),
					to: SmallU64Set::new(),
					of: of.clone(),
				};
				for child in of.iter() {
					if child != id {
						merges.insert(*child, *id);
					}
				}
				(*id, node)
			})
			.collect();
		for &(from, to) in &ipc.edges {
			let (from, to) = (ipc.metadata[from].id, ipc.metadata[to].id);
			nodes.get_mut(&from).unwrap().to.insert(to);
			nodes.get_mut(&to).unwrap().from.insert(from);
		}
		Self { nodes, merges }
	}
}
impl From<&Graph> for GraphIpc {
	fn from(graph: &Graph) -> GraphIpc {
		let mut num_edges = 0;
		let metadata: Vec<NodeMetadata> = graph
			.nodes
			.values()
			.inspect(|node| num_edges += node.to.len())
			.map(|node| NodeMetadata {
				id: node.id,
				of: node.of.clone(),
			})
			.collect();
		let id_to_idx: HashMap<u64, usize> = metadata
			.iter()
			.enumerate()
			.map(|(idx, NodeMetadata { id, .. })| (*id, idx))
			.collect();
		let mut edges = Vec::with_capacity(num_edges);
		for node in graph.nodes.values() {
			for dst_id in node.to.iter() {
				edges.push((id_to_idx[&node.id], id_to_idx[dst_id]));
			}
		}
		Self { metadata, edges }
	}
}
