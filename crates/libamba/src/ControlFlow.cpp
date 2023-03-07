#include "ControlFlow.h"

namespace control_flow {

void onBlockStart(
	s2e::S2EExecutionState *const s2e_state,
	const u64 pc,
	std::unordered_map<BlockId, Block> *const control_flow_graph,
	ControlFlowState *const cfg_state
) {
	auto &cfg = *control_flow_graph;
	auto &block = cfg[pc]; // unordered_map::[] will insert if it doesn't already exist

	block.from.insert(cfg_state->last);

	for (auto from : block.from) {
		cfg[from].to.insert(pc);
	}

	cfg_state->last = pc;
}

}
