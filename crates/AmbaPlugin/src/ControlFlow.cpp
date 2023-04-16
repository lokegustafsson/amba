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

StatePC ControlFlow::packStatePc(IdS2E uid, u64 pc) {
	return pc << 4 | (u64) uid.val;
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

Packed ControlFlow::getPacked(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const IdS2E state = IdS2E(s2e_state->getID());
	const IdAmba amba_id = this->getIdAmba(state);
	const StatePC state_pc = this->packStatePc(state, pc);
	const Generation gen = this->m_generations[state_pc];
	const u64 vaddr = pc;

	const u64 packed
		= (0x0000'FFFF'FFFF'FFFF & vaddr)
		| (0x000F'0000'0000'0000 & ((u64) gen.val << 48))
		| (0xFFF0'0000'0000'0000 & ((u64) amba_id.val << 52));

	{
		const Unpacked unpacked = control_flow::unpack(packed);
		AMBA_ASSERT(vaddr == unpacked.vaddr);
		AMBA_ASSERT(gen == unpacked.gen);
		AMBA_ASSERT((u64) amba_id.val == unpacked.state);
	}

	return Packed(packed);
}


} // namespace control_flow
