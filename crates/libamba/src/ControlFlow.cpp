#include "ControlFlow.h"

namespace control_flow {

ControlFlow::ControlFlow()
	: m_last(0)
	, m_cfg(rust_new_control_flow_graph())
	{}

ControlFlow::~ControlFlow() {
	rust_free_control_flow_graph(this->m_cfg);
}

} // namespace control_flow
