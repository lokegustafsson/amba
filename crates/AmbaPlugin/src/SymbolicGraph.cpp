#include "SymbolicGraph.h"
#include "AmbaException.h"
#include "ControlFlow.h"

namespace symbolic_graph {

SymbolicGraph::SymbolicGraph(std::string name)
	: ControlFlow(name)
{}

void SymbolicGraph::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	const StateIdAmba from = this->getStateIdAmba(control_flow::getStateIdS2E(old_state));

	for (auto &new_state : new_states) {
		if (new_state == old_state) {
			this->incrementStateIdAmba(control_flow::getStateIdS2E(new_state));
		}

		const StateIdAmba to = this->getStateIdAmba(control_flow::getStateIdS2E(new_state));
		AMBA_ASSERT(from != to);

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
	const StateIdS2E dest_id = control_flow::getStateIdS2E(destination_state);
	const StateIdS2E src_id = control_flow::getStateIdS2E(source_state);

	const StateIdAmba from_left = this->getStateIdAmba(dest_id);
	const StateIdAmba from_right = this->getStateIdAmba(src_id);

	this->incrementStateIdAmba(dest_id);
	const StateIdAmba to = this->getStateIdAmba(dest_id);

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
