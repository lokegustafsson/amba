#pragma once

#include "Numbers.h"
#include "Amba.h"

struct ControlFlowGraph;

extern "C" {
	ControlFlowGraph *rust_new_control_flow_graph();
	void rust_free_control_flow_graph(ControlFlowGraph *ptr);
	void rust_print_graph_size(ControlFlowGraph *ptr);
	void rust_update_control_flow_graph(
		ControlFlowGraph *ptr,
		u64 from,
		u64 to
	);
}

namespace control_flow {

class ControlFlow {
  public:
	ControlFlow();
	~ControlFlow();

	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
  protected:
	u64 m_last;
	ControlFlowGraph *m_cfg;
};

} // namespace control_flow
