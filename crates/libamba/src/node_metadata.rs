#[repr(C)]
pub struct NodeMetadataFFI {
	pub metadata_type: u32,
	pub symbolic_state_id: u32,
	pub basic_block_vaddr: u64,
	pub basic_block_generation: u64,
	pub basic_block_elf_vaddr: u64,
	pub basic_block_content: cxx::UniquePtr<cxx::CxxVector<u32>>,
}

impl From<&NodeMetadataFFI> for ipc::NodeMetadata {
	fn from(value: &NodeMetadataFFI) -> Self {
		let &NodeMetadataFFI {
			metadata_type,
			symbolic_state_id,
			basic_block_vaddr,
			basic_block_generation,
			basic_block_elf_vaddr: _,
			basic_block_content: _,
		} = value;
		match metadata_type {
			0 => ipc::NodeMetadata::State { symbolic_state_id },
			1 => ipc::NodeMetadata::BasicBlock {
				symbolic_state_id,
				basic_block_vaddr: basic_block_vaddr.try_into().ok(),
				basic_block_generation: basic_block_generation.try_into().ok(),
			},
			_ => panic!("Invalid metadata type"),
		}
	}
}

/// An FFI-safe `(NodeMetadataFFI, NodeMetadataFFI)` tuple.  Directly
/// represents the delayed input to
/// `data_structures::ControlFlowGraph::update` which in turn
/// represents an Edge
#[repr(C)]
pub struct NodeMetadataFFIPair {
	pub fst: NodeMetadataFFI,
	pub snd: NodeMetadataFFI,
}

impl From<&NodeMetadataFFIPair> for (ipc::NodeMetadata, ipc::NodeMetadata) {
	fn from(NodeMetadataFFIPair { fst, snd }: &NodeMetadataFFIPair) -> Self {
		(fst.into(), snd.into())
	}
}
