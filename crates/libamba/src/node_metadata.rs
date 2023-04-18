#[repr(C)]
pub struct NodeMetadataFFI {
	pub symbolic_state_id: u32,
	pub basic_block_vaddr: u64,
	pub basic_block_generation: u64,
}

impl From<NodeMetadataFFI> for ipc::NodeMetadata {
	fn from(value: NodeMetadataFFI) -> Self {
		let NodeMetadataFFI {
			symbolic_state_id,
			basic_block_vaddr,
			basic_block_generation,
		} = value;
		ipc::NodeMetadata {
			symbolic_state_id,
			basic_block_vaddr: basic_block_vaddr.try_into().ok(),
			basic_block_generation: basic_block_generation.try_into().ok(),
		}
	}
}
