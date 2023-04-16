#include "ControlFlow.h"
#include "AmbaException.h"

namespace control_flow {

Unpacked unpack(Packed packed) {
	const u64 val = packed.val;
	// Addresses either live at the bottom or top of the address
	// space, so we sign extend from the 48 bits we have kept
	u64 vaddr =  val & 0x0000'FFFF'FFFF'FFFF;
	if (vaddr & 1L << 47) {
		vaddr |= 0xFFFF'0000'0000'0000;
	}
	const u64 gen   = (val & 0x000F'0000'0000'0000) >> 48;
	const u64 state = (val & 0xFFF0'0000'0000'0000) >> 52;

	return (Unpacked) {
		.vaddr = vaddr,
		.gen = (u8) gen,
		.state = state,
	};
}

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
