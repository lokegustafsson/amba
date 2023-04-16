#pragma once

#include <string>
#include <unordered_map>

#include "ControlFlow.h"

namespace assembly_graph {

using namespace control_flow::types;

void updateControlFlowGraph(ControlFlowGraph *, Packed, Packed);

class AssemblyGraph : public control_flow::ControlFlow {
  public:
	AssemblyGraph(std::string);

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
};

}
