use std::{mem, num::NonZeroU64};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

const SMALL_SIZE_COMPRESSED: usize = {
	assert!(mem::size_of::<SmallVec<[u32; 4]>>() == mem::size_of::<SmallVec<[u32; 0]>>());
	assert!(mem::size_of::<SmallVec<[u32; 5]>>() >= mem::size_of::<SmallVec<[u32; 0]>>());
	assert!(mem::size_of::<SmallVec<[u64; 4]>>() == mem::size_of::<[u64; 5]>());
	4
};
const SMALL_SIZE_U8: usize = {
	assert!(mem::size_of::<SmallVec<[u8; 16]>>() == mem::size_of::<SmallVec<[u8; 0]>>());
	assert!(mem::size_of::<SmallVec<[u8; 17]>>() >= mem::size_of::<SmallVec<[u8; 0]>>());
	16
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeMetadata {
	State {
		amba_state_id: u32,
	},
	BasicBlock {
		symbolic_state_id: u32,
		basic_block_vaddr: Option<NonZeroU64>,
		basic_block_generation: Option<NonZeroU64>,
		basic_block_elf_vaddr: Option<NonZeroU64>,
		basic_block_content: SmallVec<[u8; SMALL_SIZE_U8]>,
	},
	CompressedBasicBlock(Box<CompressedBasicBlock>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompressedBasicBlock {
	pub symbolic_state_ids: SmallVec<[u32; SMALL_SIZE_COMPRESSED]>,
	pub basic_block_vaddrs: SmallVec<[Option<NonZeroU64>; SMALL_SIZE_COMPRESSED]>,
	pub basic_block_generations: SmallVec<[Option<NonZeroU64>; SMALL_SIZE_COMPRESSED]>,
	pub basic_block_elf_vaddrs: SmallVec<[Option<NonZeroU64>; SMALL_SIZE_COMPRESSED]>,
	pub basic_block_contents: SmallVec<[SmallVec<[u8; SMALL_SIZE_U8]>; SMALL_SIZE_COMPRESSED]>,
}
