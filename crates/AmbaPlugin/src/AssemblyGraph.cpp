#include "AssemblyGraph.h"
#include "AmbaException.h"

namespace assembly_graph {

void updateControlFlowGraph(ControlFlowGraph *cfg, Packed from, Packed to) {
	rust_update_control_flow_graph(
		cfg,
		from.val,
		to.val
	);
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
		control_flow::getIdS2E(state),
		pc
	);
	++this->m_generations[key].val;
}

void AssemblyGraph::onBlockStart(
	s2e::S2EExecutionState *state,
	u64 pc
) {
	const IdAmba amba_id = this->getIdAmba(control_flow::getIdS2E(state));
	const Packed curr = this->getPacked(state, pc);
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
	this->incrementIdAmba(control_flow::getIdS2E(old_state));
}

void AssemblyGraph::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	this->incrementIdAmba(control_flow::getIdS2E(destination_state));
}

}
