#include "ControlFlow.h"
#include "AmbaException.h"

namespace control_flow {

StateIdS2E getStateIdS2E(s2e::S2EExecutionState *state) {
	return StateIdS2E(state->getGuid());
}

ControlFlow::ControlFlow(std::string name)
	: m_name(name)
	, m_cfg(rust_new_control_flow_graph())
{}

ControlFlow::~ControlFlow() {
	rust_print_graph_size(this->m_name.c_str(), this->m_cfg);
	rust_free_control_flow_graph(this->m_cfg);
}

const char *ControlFlow::getName() const {
	return this->m_name.c_str();
}

ControlFlowGraph *ControlFlow::cfg() {
	return this->m_cfg;
}

StateIdAmba ControlFlow::getStateIdAmba(StateIdS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id == 0) {
		this->next_id++;
		amba_id.val = this->next_id;
	}
	return amba_id;
}

void ControlFlow::incrementStateIdAmba(StateIdS2E id) {
	this->next_id++;
	auto& amba_id = this->m_states[id];
	amba_id.val = this->next_id;
}

} // namespace control_flow
