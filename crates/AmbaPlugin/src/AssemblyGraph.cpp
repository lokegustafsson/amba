#include "AssemblyGraph.h"
#include "AmbaException.h"
#include "ControlFlow.h"

namespace assembly_graph {

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
	const Metadata curr = this->getMetadata(state, pc);
	// Will insert 0 if value doesn't yet exist
	Metadata &last = this->m_last[amba_id];
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
	const StateIdAmba old_amba_id = this->getStateIdAmba(control_flow::getStateIdS2E(old_state));
	const Metadata old_last = this->m_last[old_amba_id];

	this->incrementStateIdAmba(control_flow::getStateIdS2E(old_state));

	for (auto& new_state : new_states) {
		const StateIdAmba new_amba_id = this->getStateIdAmba(control_flow::getStateIdS2E(new_state));
		this->m_last[new_amba_id] = old_last;

	}
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

Metadata AssemblyGraph::getMetadata(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const StateIdS2E state = StateIdS2E(s2e_state->getID());
	const StateIdAmba amba_id = this->getStateIdAmba(state);
	const StatePC state_pc = this->packStatePc(state, pc);
	const BasicBlockGeneration gen = this->m_generations[state_pc];

	return (Metadata) {
		.symbolic_state_id = amba_id,
		.basic_block_vaddr = pc,
		.basic_block_generation = gen.val,
	};
}

}
