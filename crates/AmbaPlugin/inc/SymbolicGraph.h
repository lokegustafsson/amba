#pragma once

#include <string>
#include <unordered_map>

#include "ControlFlow.h"

namespace symbolic_graph {

using namespace control_flow::types;

void updateControlFlowGraph(ControlFlowGraph *, AmbaUid, AmbaUid);

class SymbolicGraph : public control_flow::ControlFlow {
  public:
	SymbolicGraph(std::string);

	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
};

}
