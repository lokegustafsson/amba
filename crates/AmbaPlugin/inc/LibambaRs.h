#include "Numbers.h"

struct ControlFlowGraph;
struct IpcTx;
struct IpcRx;

struct NodeMetadataFFI {
	u32 symbolic_state_id;
	u64 basic_block_vaddr;
	u64 basic_block_generation;
};

struct NodeMetadataFFIPair {
	NodeMetadataFFI fst;
	NodeMetadataFFI snd;
};

extern "C" {
	IpcTx *rust_new_ipc();
	void rust_free_ipc(IpcTx *ptr);
	void rust_ipc_send_graphs(
		IpcTx *ipc,
		ControlFlowGraph *symbolic,
		ControlFlowGraph *assembly
	);
	void rust_ipc_send_edges(
		IpcTx *ipc,
		const NodeMetadataFFIPair *state_data,
		u64 state_len,
		const NodeMetadataFFIPair *block_data,
		u64 block_len
	);
}
