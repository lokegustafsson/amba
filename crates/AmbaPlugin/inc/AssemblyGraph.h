#pragma once

#include <string>

#include "ControlFlow.h"

namespace assembly_graph {

using namespace control_flow::types;

Unpacked unpack(PackedNodeData);

class AssemblyGraph : public control_flow::ControlFlow {
  public:
	AssemblyGraph(std::string);

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;

  protected:
	StatePC packStatePc(StateIdS2E, u64);
	Metadata getMetadata(s2e::S2EExecutionState *, u64);

	std::unordered_map<StatePC, BasicBlockGeneration> m_generations {};
	std::unordered_map<StateIdAmba, Metadata> m_last {};
};

}
