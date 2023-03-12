#pragma once

#include "S2EForwardDeclarations.h"
#include "Numbers.h"

extern "C" {
	struct ControlFlowGraph;
	ControlFlowGraph *rust_create_control_flow_graph();
	void rust_free_control_flow_graph(ControlFlowGraph *);
}

namespace control_flow {

class ControlFlow {
  protected:
	ControlFlowGraph *m_cfg;

  public:
	ControlFlow ();
	~ControlFlow ();

	void onBlockStart(
		s2e::S2EExecutionState *s2e_state,
		u64 pc
	);
};

}
