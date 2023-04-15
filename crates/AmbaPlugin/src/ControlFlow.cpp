#include "ControlFlow.h"
#include "AmbaException.h"

namespace control_flow {

void updateControlFlowGraph(ControlFlowGraph *cfg, Packed from, Packed to) {
	rust_update_control_flow_graph(
		cfg,
		from.val,
		to.val
	);
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

} // namespace control_flow
