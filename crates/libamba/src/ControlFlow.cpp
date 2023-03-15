#include "ControlFlow.h"

namespace control_flow {

ControlFlow::ControlFlow()
	: m_last(0)
	, m_cfg(rust_new_control_flow_graph())
	{}

ControlFlow::~ControlFlow() {
	rust_free_control_flow_graph(this->m_cfg);
}

void ControlFlow::onBlockStart(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	rust_update_control_flow_graph(
		this->m_cfg,
		this->m_last,
		pc
	);
	this->m_last = pc;
}
} // namespace control_flow
