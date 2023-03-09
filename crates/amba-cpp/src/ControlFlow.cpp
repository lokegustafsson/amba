#include "ControlFlow.h"

namespace control_flow {

void ControlFlow::onBlockStart(
	s2e::S2EExecutionState *const s2e_state,
	const u64 pc
) {
	auto &block = this->m_cfg[pc]; // unordered_map::[] will insert if it doesn't already exist

	block.from.insert(this->last);

	for (auto from : block.from) {
		this->m_cfg[from].to.insert(pc);
	}

	this->last = pc;
}

}
