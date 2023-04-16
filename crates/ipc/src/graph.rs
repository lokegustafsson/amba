use std::{
	collections::{HashMap, HashSet},
	num::NonZeroU64,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeMetadata {
	pub symbolic_state_id: u32,
	pub basic_block_vaddr: Option<NonZeroU64>,
	pub basic_block_generation: Option<NonZeroU64>,
}

impl From<u32> for NodeMetadata {
	fn from(value: u32) -> Self {
		Self {
			symbolic_state_id: value,
			basic_block_vaddr: None,
			basic_block_generation: None,
		}
	}
}

impl NodeMetadata {
	fn pack(&self) -> u64 {
		let Self {
			symbolic_state_id: state,
			basic_block_vaddr: vaddr,
			basic_block_generation: gen,
		} = *self;
		let vaddr = vaddr.map_or(0, NonZeroU64::get);
		let gen = gen.map_or(0, NonZeroU64::get);
		let state = state as u64;

		// Pack bits
		let mut ret = 0;
		ret |= 0x0000_FFFF_FFFF_FFFF & vaddr;
		ret |= 0x000F_0000_0000_0000 & (gen << 48);
		ret |= 0xFFF0_0000_0000_0000 & (state << (48 + 4));

		ret
	}

	fn unpack(val: u64) -> Self {
		let vaddr = {
			let base = 0x0000_FFFF_FFFF_FFFF & val;
			// Sign extend from 48 bits
			if val & (1 << 47) != 0 {
				base | (0xFFFF << 48)
			} else {
				base
			}
		};
		let gen = (0x000F_0000_0000_0000 & val) >> 48;
		let state = (0xFFF0_0000_0000_0000 & val) >> (48 + 4);

		let symbolic_state_id = state as u32;
		let basic_block_vaddr = vaddr.try_into().ok();
		let basic_block_generation = gen.try_into().ok();

		Self {
			symbolic_state_id,
			basic_block_vaddr,
			basic_block_generation,
		}
	}

	pub fn unique_id(&self) -> u64 {
		let ret = self.pack();
		let unpacked = Self::unpack(ret);

		// Assert that the packing is injective, no data was lost
		assert_eq!(*self, unpacked);

		ret
	}
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
