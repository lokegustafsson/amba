#pragma once

#include <string>
#include <unordered_map>

#include "ControlFlow.h"

namespace symbolic_graph {

using namespace control_flow::types;

class SymbolicGraph : public control_flow::ControlFlow {
  public:
	SymbolicGraph(std::string);

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;

  protected:
	StatePC toAlias(UidS2E, u64);
	Packed getBlockId(s2e::S2EExecutionState *, u64);

	/// State uuid → reuses
	std::unordered_map<UidS2E, Packed> m_uuids {};

	/// (State, pc) → gen
	std::unordered_map<StatePC, Generation> m_generations {};

	/// Either:
	/// State → (State, pc)
	/// Alias → Alias
	std::unordered_map<u64, u64> m_last {};
};

}
