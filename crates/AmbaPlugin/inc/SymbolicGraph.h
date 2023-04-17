#pragma once

#include <string>

#include "ControlFlow.h"

namespace symbolic_graph {

using namespace control_flow::types;

class SymbolicGraph : public control_flow::ControlFlow {
  public:
	SymbolicGraph(std::string);

	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;
};

}
