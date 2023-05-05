#pragma once

#include <vector>
#include <memory>

#include "Numbers.h"

struct ControlFlowGraph;
struct Ipc;

struct NodeMetadataFFI {
	u32 metadata_type;
	u32 amba_state_id;
	i32 s2e_state_id;
	u64 basic_block_vaddr;
	u64 basic_block_generation;
	u64 basic_block_elf_vaddr;
	std::unique_ptr<std::vector<u8>> basic_block_content;
};

struct NodeMetadataFFIPair {
	NodeMetadataFFI fst;
	NodeMetadataFFI snd;
};

extern "C" {
	Ipc *rust_new_ipc();
	void rust_free_ipc(Ipc *ptr);
	void rust_ipc_send_edges(
		Ipc *ipc,
		const NodeMetadataFFIPair *state_data,
		u64 state_len,
		const NodeMetadataFFIPair *block_data,
		u64 block_len
	);
	bool rust_ipc_receive_message(Ipc *ipc, std::vector<u32> *vec);
}
