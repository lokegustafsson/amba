#pragma once

#include <unordered_map>

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
	u64 m_last_uuid = 0;
	std::unordered_map<u32, u32> m_uuids {};
	ControlFlowGraph *m_cfg;
};

} // namespace control_flow
