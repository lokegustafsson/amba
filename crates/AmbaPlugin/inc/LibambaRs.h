#pragma once

#include <vector>
#include <memory>

#include "Numbers.h"

struct IpcTx;
struct IpcRx;

struct IpcPair {
	IpcTx *tx;
	IpcRx *rx;
};

struct NodeMetadataFFI {
	u32 metadata_type;
	u32 symbolic_state_id;
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
	IpcPair rust_new_ipc();
	void rust_free_ipc(IpcPair *ptr);
	void rust_ipc_send_edges(
		IpcTx *ipc,
		const NodeMetadataFFIPair *state_data,
		u64 state_len,
		const NodeMetadataFFIPair *block_data,
		u64 block_len
	);
}
