#include "Numbers.h"

struct ControlFlowGraph;

extern "C" {
	ControlFlowGraph *rust_new_control_flow_graph();
	void rust_free_control_flow_graph(ControlFlowGraph *ptr);
	void rust_update_control_flow_graph(
		ControlFlowGraph *ptr,
		u64 from,
		u64 to
	);
	void rust_print_graph_size(const char *name, ControlFlowGraph *ptr);
	void rust_ipc_send_graph(const char *name, ControlFlowGraph *graph);
}
