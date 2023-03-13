#pragma once

#include <s2e/S2EExecutionState.h>

#include <unordered_map>
#include <unordered_set>

#include "Numbers.h"

namespace control_flow {

using BlockId = u64;

struct ControlFlowState {
	BlockId last;
};

struct Block {
	BlockId id;
	std::unordered_set<BlockId> from;
	std::unordered_set<BlockId> to;
};

void onBlockStart(
	s2e::S2EExecutionState *s2e_state,
	u64 pc,
	std::unordered_map<BlockId, Block> *control_flow_graph,
	ControlFlowState *cfg_state
);

}
