use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeMetadata {
	pub symbolic_state_id: u32,
	pub basic_block_vaddr: Option<NonZeroU64>,
	pub basic_block_generation: Option<NonZeroU64>,
}
