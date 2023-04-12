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
	u64 getBlockId(s2e::S2EExecutionState *, u64);

	const std::string m_name;
	ControlFlowGraph *const m_cfg;

	/// State uuid → reuses
	std::unordered_map<i32, u64> m_uuids {};

	/// (State, pc) → gen
	std::unordered_map<u64, u64> m_generations {};

	/// Either:
	/// State → (State, pc)
	/// Alias → Alias
	std::unordered_map<u64, u64> m_last {};
};

} // namespace control_flow
