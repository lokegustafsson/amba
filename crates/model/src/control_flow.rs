use std::{
	collections::HashMap,
	fmt,
	time::{Duration, Instant},
};

use data_structures::Graph;
use ipc::{CompressedBasicBlock, NodeMetadata};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
	pub graph: Graph,
	pub compressed_graph: Graph,
	pub(crate) updates: usize,
	pub(crate) rebuilds: usize,
	pub(crate) created_at: Instant,
	pub(crate) rebuilding_time: Duration,
	pub metadata: Vec<NodeMetadata>,
	meta_mapping_unique_id_to_index: HashMap<NodeMetadata, usize>,
}

impl FromIterator<(NodeMetadata, NodeMetadata)> for ControlFlowGraph {
	fn from_iter<T: IntoIterator<Item = (NodeMetadata, NodeMetadata)>>(iter: T) -> Self {
		let mut ret = ControlFlowGraph::new();
		for (from, to) in iter.into_iter() {
			ret.update(from, to);
		}
		ret
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
			meta_mapping_unique_id_to_index: HashMap::new(),
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
		*self
			.meta_mapping_unique_id_to_index
			.entry(node.clone())
			.or_insert_with(|| {
				let seq_index = self.metadata.len();
				self.metadata.push(node);
				seq_index
			}) as u64
	}

	pub fn get_raw_metadata_and_selfedge_and_sequential_edges(
		&self,
	) -> (Vec<NodeMetadata>, Vec<bool>, Vec<(usize, usize)>) {
		(
			self.metadata.clone(),
			(0..(self.metadata.len() as u64))
				.map(|idx| self.graph.nodes[&idx].to.contains(&idx))
				.collect(),
			self.graph
				.edges()
				.map(|(from, to)| (from as usize, to as usize))
				.collect(),
		)
	}

	pub fn get_compressed_metadata_and_selfedge_and_sequential_edges(
		&self,
	) -> (Vec<NodeMetadata>, Vec<bool>, Vec<(usize, usize)>) {
		// NOTE: Iterating in increasing-id-order over `self.nodes` is crucial for
		// correctness (here guaranteed by BTreeMap).
		let node_id_renamings = self
			.compressed_graph
			.nodes
			.keys()
			.copied()
			.enumerate()
			.map(|(x, y)| (y, x))
			.collect::<HashMap<_, _>>();
		(
			self.compressed_graph
				.nodes
				.iter()
				.enumerate()
				.map(|(i, (id, node))| {
					assert_eq!(node_id_renamings[id], i);
					merge_nodes_into_single_metadata(
						node.of
							.iter()
							.map(|component_node| &self.metadata[*component_node as usize]),
					)
				})
				.collect(),
			self.compressed_graph
				.nodes
				.iter()
				.map(|(id, node)| node.to.contains(id))
				.collect(),
			self.compressed_graph
				.edges()
				.map(|(from, to)| (node_id_renamings[&from], node_id_renamings[&to]))
				.collect(),
		)
	}
}

fn merge_nodes_into_single_metadata<'a>(
	raw_nodes: impl Iterator<Item = &'a NodeMetadata>,
) -> NodeMetadata {
	let mut symbolic_state_ids = SmallVec::new();
	let mut basic_block_vaddrs = SmallVec::new();
	let mut basic_block_generations = SmallVec::new();
	let mut basic_block_elf_vaddrs = SmallVec::new();
	let mut basic_block_contents = SmallVec::new();

	for metadata in raw_nodes {
		if let NodeMetadata::BasicBlock {
			symbolic_state_id,
			basic_block_vaddr,
			basic_block_generation,
			basic_block_elf_vaddr,
			basic_block_content,
		} = metadata
		{
			symbolic_state_ids.push(*symbolic_state_id);
			basic_block_vaddrs.push(*basic_block_vaddr);
			basic_block_generations.push(*basic_block_generation);
			basic_block_elf_vaddrs.push(*basic_block_elf_vaddr);
			basic_block_contents.push(basic_block_content.clone());
		} else {
			panic!("Basic block graph contained non-basic-block metadata")
		};
	}

	NodeMetadata::CompressedBasicBlock(Box::new(CompressedBasicBlock {
		symbolic_state_ids,
		basic_block_vaddrs,
		basic_block_generations,
		basic_block_elf_vaddrs,
		basic_block_contents,
	}))
}

#[cfg(test)]
mod test {
	use crate::control_flow::*;

	fn node(i: u32) -> NodeMetadata {
		NodeMetadata::State {
			amba_state_id: i,
			s2e_state_id: i,
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
