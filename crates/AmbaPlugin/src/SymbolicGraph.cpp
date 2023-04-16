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
