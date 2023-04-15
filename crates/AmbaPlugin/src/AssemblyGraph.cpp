#include "AssemblyGraph.h"
#include "ControlFlow.h"
#include "AmbaException.h"

namespace assembly_graph {

AssemblyGraph::AssemblyGraph(std::string name)
	: ControlFlow(name)
{}

StatePC AssemblyGraph::toAlias(UidS2E uid, u64 pc) {
	return pc << 4 | (u64) this->m_uuids[uid].val;
}

Packed AssemblyGraph::getBlockId(
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

	{
		// Addresses either live at the bottom or top of the address
		// space, so we sign extend from the 48 bits we have kept
		u64 vaddr_ =  packed & 0x0000'FFFF'FFFF'FFFF;
		if (vaddr & 1L << 47) {
			vaddr_ |= 0xFFFF'0000'0000'0000;
		}
		AMBA_ASSERT(vaddr == vaddr_);

		const u64 gen_   = (packed & 0x000F'0000'0000'0000) >> 48;
		AMBA_ASSERT(gen.val == gen_);

		const u64 state_ = (packed & 0xFFF0'0000'0000'0000) >> 52;
		AMBA_ASSERT((u64) state.val == state_);
	}

	return Packed(packed);
}

void AssemblyGraph::translateBlockStart(
	s2e::ExecutionSignal *signal,
	s2e::S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	const StatePC key = this->toAlias(state->getID(), pc);
	++this->m_generations[key].val;
}

void AssemblyGraph::onBlockStart(
	s2e::S2EExecutionState *state,
	u64 pc
) {
	const Packed curr = this->getBlockId(state, pc);
	// Will insert 0 if value doesn't yet exist
	auto &last = this->m_last[curr];
	control_flow::updateControlFlowGraph(
		this->m_cfg,
		last,
		curr
	);
	last = curr;
}
	
}
