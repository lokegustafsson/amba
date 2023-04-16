#pragma once

#include <string>
#include <unordered_map>

#include "ControlFlow.h"

namespace assembly_graph {

using namespace control_flow::types;

Unpacked unpack(Packed packed);

class AssemblyGraph : public control_flow::ControlFlow {
  public:
	AssemblyGraph(std::string);

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;

  protected:
	StatePC toAlias(UidS2E, u64);
	Packed getPacked(s2e::S2EExecutionState *, u64);
	AmbaUid getAmbaId(UidS2E);
	void incrementAmbaId(UidS2E);

	std::unordered_map<UidS2E, AmbaUid> m_states {};
	std::unordered_map<StatePC, Generation> m_generations {};
	std::unordered_map<AmbaUid, Packed> m_last {};
};

}
