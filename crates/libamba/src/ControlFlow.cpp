#include "Amba.h"
#include "ControlFlow.h"

namespace control_flow {

ControlFlow::ControlFlow() : m_cfg(rust_create_control_flow_graph()) {}

ControlFlow::~ControlFlow() {
	rust_free_control_flow_graph(this->m_cfg);
}

void ControlFlow::onBlockStart(
	s2e::S2EExecutionState *const s2e_state,
	const u64 pc
) {
	/*
	auto &block = this->m_cfg[pc]; // unordered_map::[] will insert if it doesn't already exist

	block.from.insert(this->last);

	for (auto from : block.from) {
		this->m_cfg[from].to.insert(pc);
	}

	this->last = pc;
	*/
}

}
