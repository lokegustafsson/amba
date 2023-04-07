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
	std::string m_name;

	u64 m_last_uuid = 0;
	std::unordered_map<i32, u64> m_uuids {};

	ControlFlowGraph *m_cfg;
	std::unordered_map<u64, u64> m_last = {};
};

} // namespace control_flow
