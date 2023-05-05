#include <vector>
#include <tuple>

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
	const auto s2e_state_id = control_flow::getStateIdS2E(old_state);
	const auto amba_state_id = this->getStateIdAmba(s2e_state_id);
	const auto from = (StateMetadata) {
		.amba_state_id = amba_state_id,
		.s2e_state_id = s2e_state_id,
	};

	this->incrementStateIdAmba(control_flow::getStateIdS2E(old_state));
	for (auto &new_state : new_states) {
		const auto to_s2e_state_id = control_flow::getStateIdS2E(new_state);
		const auto to_amba_state_id = this->getStateIdAmba(to_s2e_state_id);
		const auto to = (StateMetadata) {
			.amba_state_id = to_amba_state_id,
			.s2e_state_id = to_s2e_state_id,
		};
		AMBA_ASSERT(from.amba_state_id != to.amba_state_id);

		this->m_new_edges.push_back(
			(NodeMetadataFFIPair) {
				.fst = from.into_ffi(),
				.snd = to.into_ffi()
			}
		);
	}
}

void SymbolicGraph::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	const StateIdS2E dest_id = control_flow::getStateIdS2E(destination_state);
	const StateIdS2E src_id = control_flow::getStateIdS2E(source_state);

	const auto from_left = (StateMetadata) {
		.amba_state_id = this->getStateIdAmba(dest_id),
		.s2e_state_id = dest_id,
	};
	const auto from_right = (StateMetadata) {
		.amba_state_id = this->getStateIdAmba(src_id),
		.s2e_state_id = src_id,
	};

	this->incrementStateIdAmba(dest_id);
	const auto to = (StateMetadata) {
		.amba_state_id = this->getStateIdAmba(dest_id),
		.s2e_state_id = dest_id,
	};

	this->m_new_edges.push_back(
		(NodeMetadataFFIPair) {
			.fst = from_left.into_ffi(),
			.snd = to.into_ffi()
		}
	);
	this->m_new_edges.push_back(
		(NodeMetadataFFIPair) {
			.fst = from_right.into_ffi(),
			.snd = to.into_ffi()
		}
	);
}

}
