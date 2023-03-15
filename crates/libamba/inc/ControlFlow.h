#pragma once

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
}

namespace control_flow {

class ControlFlow {
  public:
	ControlFlow();
	~ControlFlow();
  protected:
	ControlFlowGraph *m_cfg;
};

} // namespace control_flow
