#include "SymbolicGraph.h"
#include "ControlFlow.h"
#include "AmbaException.h"

namespace symbolic_graph {

void updateControlFlowGraph(ControlFlowGraph *cfg, AmbaUid from, AmbaUid to) {
	rust_update_control_flow_graph(
		cfg,
		from.val,
		to.val
	);
}

SymbolicGraph::SymbolicGraph(std::string name)
	: ControlFlow(name)
{}

void SymbolicGraph::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	const AmbaUid from = this->getAmbaId(control_flow::getID(old_state));
	const auto last_raw = this->m_last[from.val];

	for (auto &new_state : new_states) {
		if (new_state == old_state) {
			this->incrementAmbaId(control_flow::getID(old_state));
		}

		const AmbaUid to = this->getAmbaId(control_flow::getID(old_state));
		this->m_last[to] = last_raw;

		updateControlFlowGraph(
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
	const UidS2E dest_id = control_flow::getID(destination_state);
	const UidS2E src_id = control_flow::getID(source_state);

	const AmbaUid from_left = this->getAmbaId(dest_id);
	const AmbaUid from_right = this->getAmbaId(src_id);

	this->incrementAmbaId(dest_id);
	const AmbaUid to = this->getAmbaId(dest_id);

	updateControlFlowGraph(
		this->m_cfg,
		from_left,
		to
	);
	updateControlFlowGraph(
		this->m_cfg,
		from_right,
		to
	);
}

}
