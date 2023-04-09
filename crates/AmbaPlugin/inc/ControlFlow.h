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

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
	amba::TimerFunction onTimer;
	amba::TimerFunction onEngineShutdown;
  protected:
	const std::string m_name;

	u64 m_last_uuid = 0;
	std::unordered_map<i32, u64> m_uuids {};

	std::unordered_map<u64, u64> m_last = {};
	ControlFlowGraph *const m_cfg;
};

} // namespace control_flow
