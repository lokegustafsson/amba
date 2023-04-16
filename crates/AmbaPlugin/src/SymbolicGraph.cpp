#include "SymbolicGraph.h"
#include "AmbaException.h"

namespace symbolic_graph {

void updateControlFlowGraph(ControlFlowGraph *cfg, IdAmba from, IdAmba to) {
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
	const IdAmba from = this->getIdAmba(control_flow::getIdS2E(old_state));
	const Packed last_raw = this->m_last[from.val];

	for (auto &new_state : new_states) {
		if (new_state == old_state) {
			this->incrementIdAmba(control_flow::getIdS2E(old_state));
		}

		const IdAmba to = this->getIdAmba(control_flow::getIdS2E(old_state));
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
	const IdS2E dest_id = control_flow::getIdS2E(destination_state);
	const IdS2E src_id = control_flow::getIdS2E(source_state);

	const IdAmba from_left = this->getIdAmba(dest_id);
	const IdAmba from_right = this->getIdAmba(src_id);

	this->incrementIdAmba(dest_id);
	const IdAmba to = this->getIdAmba(dest_id);

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
