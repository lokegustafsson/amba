#include "Numbers.h"

struct ControlFlowGraph;
struct IpcTx;
struct IpcRx;

struct NodeMetadataFFI {
	u32 symbolic_state_id;
	u64 basic_block_vaddr;
	u64 basic_block_generation;
};

extern "C" {
	ControlFlowGraph *rust_new_control_flow_graph();
	void rust_free_control_flow_graph(ControlFlowGraph *ptr);
	void rust_update_control_flow_graph(
		ControlFlowGraph *ptr,
		NodeMetadataFFI from,
		NodeMetadataFFI to
	);
	void rust_print_graph_size(
		const char *name,
		ControlFlowGraph *ptr
	);

	IpcTx *rust_new_ipc();
	void rust_free_ipc(IpcTx *ptr);
	void rust_ipc_send_graphs(
		IpcTx *ipc,
		ControlFlowGraph *symbolic,
		ControlFlowGraph *assembly
	);
}
