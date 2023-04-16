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

UidS2E getID(s2e::S2EExecutionState *state) {
	return UidS2E(state->getID());
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

StatePC ControlFlow::toAlias(UidS2E uid, u64 pc) {
	return pc << 4 | (u64) uid.val;
}

AmbaUid ControlFlow::getAmbaId(UidS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id.val == 0) {
		amba_id.val = (u64) id.val;
	}
	return amba_id;
}

void ControlFlow::incrementAmbaId(UidS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id.val == 0) {
		amba_id.val = (u64) id.val;
	}
	++amba_id.val;
}

Packed ControlFlow::getPacked(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const UidS2E state = UidS2E(s2e_state->getID());
	const AmbaUid amba_id = this->getAmbaId(state);
	const StatePC state_pc = this->toAlias(state, pc);
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
