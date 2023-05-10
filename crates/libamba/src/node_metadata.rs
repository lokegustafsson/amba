use std::num::NonZeroU64;

#[repr(C)]
pub struct NodeMetadataFFI {
	pub metadata_type: u32,
	pub amba_state_id: u32,
	pub s2e_state_id: i32,
	pub basic_block_vaddr: u64,
	pub basic_block_generation: u64,
	pub basic_block_elf_vaddr: u64,
	pub basic_block_content: cxx::UniquePtr<cxx::CxxVector<u8>>,
}

impl From<&NodeMetadataFFI> for ipc::NodeMetadata {
	fn from(
		&NodeMetadataFFI {
			metadata_type,
			amba_state_id,
			s2e_state_id,
			basic_block_vaddr,
			basic_block_generation,
			basic_block_elf_vaddr,
			ref basic_block_content,
		}: &NodeMetadataFFI,
	) -> Self {
		match metadata_type {
			0 => ipc::NodeMetadata::State {
				amba_state_id,
				s2e_state_id,
			},
			1 => ipc::NodeMetadata::BasicBlock {
				symbolic_state_id: amba_state_id,
				basic_block_vaddr: NonZeroU64::new(basic_block_vaddr),
				basic_block_generation: NonZeroU64::new(basic_block_generation),
				basic_block_elf_vaddr: NonZeroU64::new(basic_block_elf_vaddr),
				basic_block_content: basic_block_content.iter().copied().collect(),
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
