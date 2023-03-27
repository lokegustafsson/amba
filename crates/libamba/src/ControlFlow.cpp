#include "ControlFlow.h"

namespace control_flow {

ControlFlow::ControlFlow()
	: m_last(0)
	, m_cfg(rust_new_control_flow_graph())
	, m_ipc(rust_ipc_new())
	{}

ControlFlow::~ControlFlow() {
	rust_print_graph_size(this->m_cfg);
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

void ControlFlow::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	const auto old_id = old_state->getID();

	for (auto &new_state : new_states) {
		const auto new_id = new_state->getID();

		rust_update_control_flow_graph(
			this->m_cfg,
			(u64) old_id,
			(u64) new_id
		);
	}
}

void ControlFlow::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	const auto dest_id = destination_state->getID();
	const auto src_id = source_state->getID();

	rust_update_control_flow_graph(
		this->m_cfg,
		(u64) src_id,
		(u64) dest_id
	);
}

void ControlFlow::onTimer() {
	rust_ipc_send_graph(this->m_ipc, this->m_cfg);
}

} // namespace control_flow
