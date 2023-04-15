#include "SymbolicGraph.h"
#include "ControlFlow.h"
#include "AmbaException.h"

namespace symbolic_graph {

SymbolicGraph::SymbolicGraph(std::string name)
	: ControlFlow(name)
{}

StatePC SymbolicGraph::toAlias(UidS2E uid, u64 pc) {
	return this->m_uuids[uid].val << 32 | (u64) uid.val;
}

Packed SymbolicGraph::getBlockId(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const UidS2E state = UidS2E(s2e_state->getID());
	const StatePC state_pc = this->toAlias(state, pc);
	const Generation gen = this->m_generations[state_pc];
	const u64 vaddr = pc;

	const u64 packed
		= (0x0000'FFFF'FFFF'FFFF & vaddr)
		| (0x000F'0000'0000'0000 & ((u64) gen.val << 48))
		| (0xFFF0'0000'0000'0000 & ((u64) state.val << 52));

	const u64 vaddr_ =  packed & 0x0000'FFFF'FFFF'FFFF;
	const u64 gen_   = (packed & 0x000F'0000'0000'0000) >> 48;
	const u64 state_ = (packed & 0xFFF0'0000'0000'0000) >> 52;

	AMBA_ASSERT(vaddr == vaddr_);
	AMBA_ASSERT(gen.val == gen_);
	AMBA_ASSERT((u64) state.val == state_);

	return Packed(packed);
}

void SymbolicGraph::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	const UidS2E old_id = UidS2E(old_state->getID());

	const Packed from = this->getBlockId(old_state, 0);
	const u64 last_raw = this->m_last[from.val];

	for (auto &new_state : new_states) {
		const UidS2E new_id = UidS2E(new_state->getID());

		if (new_id == old_id) {
			++this->m_uuids[new_id].val;
		}

		const Packed to = this->getBlockId(new_state, 0);
		this->m_last[to.val] = last_raw;

		control_flow::updateControlFlowGraph(
			this->m_cfg,
			from,
			to
		);
	}
}

void SymbolicGraph::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	const UidS2E dest_id = UidS2E(destination_state->getID());

	const Packed from_left = this->getBlockId(destination_state, 0);
	const Packed from_right = this->getBlockId(source_state, 0);

	++this->m_uuids[dest_id].val;
	const Packed to = this->getBlockId(destination_state, 0);

	control_flow::updateControlFlowGraph(
		this->m_cfg,
		from_left,
		to
	);
	control_flow::updateControlFlowGraph(
		this->m_cfg,
		from_right,
		to
	);
}

}
