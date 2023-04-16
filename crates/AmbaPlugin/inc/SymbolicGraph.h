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
};

}
