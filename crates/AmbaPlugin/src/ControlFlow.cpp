#include "ControlFlow.h"
#include "AmbaException.h"

namespace control_flow {

IdS2E getIdS2E(s2e::S2EExecutionState *state) {
	return IdS2E(state->getGuid());
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

IdAmba ControlFlow::getIdAmba(IdS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id.val == 0) {
		amba_id.val = this->next_id;
		this->next_id++;
	}
	return amba_id;
}

void ControlFlow::incrementIdAmba(IdS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id.val == 0) {
		amba_id.val = this->next_id;
	}
	this->next_id++;
}

} // namespace control_flow
