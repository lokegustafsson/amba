#include "AssemblyGraph.h"
#include "ControlFlow.h"
#include "AmbaException.h"
#include "s2e/S2EExecutionState.h"

namespace assembly_graph {

UidS2E getID(s2e::S2EExecutionState *state) {
	return UidS2E(state->getID());
}

AssemblyGraph::AssemblyGraph(std::string name)
	: ControlFlow(name)
{}

StatePC AssemblyGraph::toAlias(UidS2E uid, u64 pc) {
	return pc << 4 | (u64) uid.val;
}

AmbaUid AssemblyGraph::getAmbaId(UidS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id.val == 0) {
		amba_id.val = (u64) id.val;
	}
	return amba_id;
}

void AssemblyGraph::incrementAmbaId(UidS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id.val == 0) {
		amba_id.val = (u64) id.val;
	}
	++amba_id.val;
}

Packed AssemblyGraph::getPacked(
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

void AssemblyGraph::translateBlockStart(
	s2e::ExecutionSignal *signal,
	s2e::S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	const StatePC key = this->toAlias(
		getID(state),
		pc
	);
	++this->m_generations[key].val;
}

void AssemblyGraph::onBlockStart(
	s2e::S2EExecutionState *state,
	u64 pc
) {
	const AmbaUid amba_id = this->getAmbaId(getID(state));
	const Packed curr = this->getPacked(state, pc);
	// Will insert 0 if value doesn't yet exist
	auto &last = this->m_last[amba_id];
	control_flow::updateControlFlowGraph(
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
	this->incrementAmbaId(getID(old_state));
}

void AssemblyGraph::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	this->incrementAmbaId(getID(destination_state));
}

}
