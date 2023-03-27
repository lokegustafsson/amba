#pragma once

#include "Numbers.h"
#include "Amba.h"
#include "LibambaRs.h"

namespace control_flow {

class ControlFlow {
  public:
	ControlFlow();
	~ControlFlow();

	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
	amba::TimerFunction onTimer;
  protected:
	u64 m_last;
	ControlFlowGraph *m_cfg;
	Ipc *m_ipc;
};

} // namespace control_flow
