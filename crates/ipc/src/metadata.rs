use std::num::NonZeroU64;

use smallvec::SmallVec;
use serde::{Deserialize, Serialize};

// Arbitrarily picked, not measured
const SMALL_SIZE: usize = 5;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeMetadata {
	State {
		symbolic_state_id: u32,
	},
	BasicBlock {
		symbolic_state_id: u32,
		basic_block_vaddr: Option<NonZeroU64>,
		basic_block_generation: Option<NonZeroU64>,
	},
	CompressedBasicBlock {
		symbolic_state_ids: SmallVec<[u32; SMALL_SIZE]>,
		basic_block_vaddrs: SmallVec<[Option<NonZeroU64>; SMALL_SIZE]>,
		basic_block_generations: SmallVec<[Option<NonZeroU64>; SMALL_SIZE]>,
	},
}
