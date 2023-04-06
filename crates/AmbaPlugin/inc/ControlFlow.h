#pragma once

#include "Numbers.h"
#include "Amba.h"
#include "LibambaRs.h"

namespace control_flow {

class ControlFlow {
  public:
	ControlFlow(std::string);
	~ControlFlow();

	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
	amba::TimerFunction onTimer;
	amba::TimerFunction onEngineShutdown;
  protected:
	u64 m_last;
  std::string m_name;
	ControlFlowGraph *m_cfg;
};

} // namespace control_flow
