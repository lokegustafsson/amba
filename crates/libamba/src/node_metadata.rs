use std::num::NonZeroU64;

use cxx::{CxxString, CxxVector};

#[repr(C)]
pub struct NodeMetadataFFI {
	pub metadata_type: u32,
	pub amba_state_id: u32,
	pub s2e_state_id: i32,
	pub basic_block_vaddr: u64,
	pub basic_block_generation: u64,
	pub basic_block_elf_vaddr: u64,
	pub basic_block_content: cxx::UniquePtr<cxx::CxxVector<u8>>,
	pub state_concrete_inputs: ConcreteInputsFFI,
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
			ref state_concrete_inputs,
		}: &NodeMetadataFFI,
	) -> Self {
		match metadata_type {
			0 => {
				let concrete_inputs = state_concrete_inputs.into();

				ipc::NodeMetadata::State {
					amba_state_id,
					s2e_state_id,
					concrete_inputs,
				}
			}
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

#[repr(C)]
pub struct ConcreteInputsFFI {
	pub names: cxx::UniquePtr<CxxVector<CxxString>>,
	pub byte_counts: cxx::UniquePtr<CxxVector<i32>>,
	pub bytes: cxx::UniquePtr<CxxVector<u8>>,
}

impl From<&ConcreteInputsFFI> for Vec<(String, Vec<u8>)> {
	fn from(
		ConcreteInputsFFI {
			names,
			byte_counts,
			bytes,
		}: &ConcreteInputsFFI,
	) -> Self {
		let mut concrete_inputs: Vec<(String, Vec<u8>)> = Vec::new();
		let bytes_slice = bytes.as_slice();
		let mut byte_index = 0;

		assert_eq!(
			bytes.len(),
			byte_counts.iter().map(|&x| x as usize).sum(),
			"{names:#?}, {bytes:#?}, {byte_counts:#?}\n"
		);

		for (i, name) in names.iter().enumerate() {
			let byte_count = *byte_counts.get(i).unwrap() as usize;
			let this_bytes = &bytes_slice[byte_index..byte_index + byte_count];
			let elem = (name.to_string(), this_bytes.to_vec());
			concrete_inputs.push(elem);
			byte_index += byte_count;
		}

		concrete_inputs
	}
}
