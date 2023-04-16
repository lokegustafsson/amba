#include "AssemblyGraph.h"
#include "AmbaException.h"

namespace assembly_graph {

void updateControlFlowGraph(ControlFlowGraph *cfg, PackedNodeData from, PackedNodeData to) {
	rust_update_control_flow_graph(
		cfg,
		from.val,
		to.val
	);
}

Unpacked unpack(PackedNodeData packed) {
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

AssemblyGraph::AssemblyGraph(std::string name)
	: ControlFlow(name)
{}

void AssemblyGraph::translateBlockStart(
	s2e::ExecutionSignal *signal,
	s2e::S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	const StatePC key = this->packStatePc(
		control_flow::getStateIdS2E(state),
		pc
	);
	++this->m_generations[key].val;
}

void AssemblyGraph::onBlockStart(
	s2e::S2EExecutionState *state,
	u64 pc
) {
	const StateIdAmba amba_id = this->getStateIdAmba(control_flow::getStateIdS2E(state));
	const PackedNodeData curr = this->getPacked(state, pc);
	// Will insert 0 if value doesn't yet exist
	auto &last = this->m_last[amba_id];
	updateControlFlowGraph(
		this->m_cfg,
		last,
		curr
	);
	last = curr;
}

void AssemblyGraph::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	this->incrementStateIdAmba(control_flow::getStateIdS2E(old_state));
}

void AssemblyGraph::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	this->incrementStateIdAmba(control_flow::getStateIdS2E(destination_state));
}

StatePC AssemblyGraph::packStatePc(StateIdS2E uid, u64 pc) {
	return pc << 4 | (u64) uid.val;
}

PackedNodeData AssemblyGraph::getPacked(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const StateIdS2E state = StateIdS2E(s2e_state->getID());
	const StateIdAmba amba_id = this->getStateIdAmba(state);
	const StatePC state_pc = this->packStatePc(state, pc);
	const BasicBlockGeneration gen = this->m_generations[state_pc];
	const u64 vaddr = pc;

	const u64 packed
		= (0x0000'FFFF'FFFF'FFFF & vaddr)
		| (0x000F'0000'0000'0000 & ((u64) gen.val << 48))
		| (0xFFF0'0000'0000'0000 & ((u64) amba_id.val << 52));

	{
		const auto [vaddr_, gen_, state_] = unpack(packed);
		AMBA_ASSERT(vaddr == vaddr_);
		AMBA_ASSERT(gen == gen_);
		AMBA_ASSERT((u64) amba_id.val == state_);
	}

	return PackedNodeData(packed);
}

}
