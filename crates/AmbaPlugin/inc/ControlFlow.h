#pragma once

#include <s2e/S2EExecutionState.h>

#include <unordered_map>
#include <vector>
#include <tuple>

#include "Numbers.h"
#include "Amba.h"
#include "LibambaRs.h"
#include "HashableWrapper.h"

namespace control_flow {

namespace types {

using StateIdS2E = hashable_wrapper::HashableWrapper<i32, 0>;
using StateIdAmba = hashable_wrapper::HashableWrapper<u64, 1>;
using StatePC = hashable_wrapper::HashableWrapper<u64, 2>;
using BasicBlockGeneration = hashable_wrapper::HashableWrapper<u8, 3>;

struct Metadata {
	StateIdAmba symbolic_state_id;
	u64 basic_block_vaddr;
	u64 basic_block_generation;

	NodeMetadataFFI into_ffi() const;
};

}

using namespace types;

StateIdS2E getStateIdS2E(s2e::S2EExecutionState *);
void updateControlFlowGraph(ControlFlowGraph *cfg, Metadata from, Metadata to);

class ControlFlow {
  public:
	ControlFlow(std::string);
	~ControlFlow();

	const char *getName() const;
	ControlFlowGraph *cfg();
	u64 states() const;
	std::vector<NodeMetadataFFIPair> &edges();

  protected:
	StateIdAmba getStateIdAmba(StateIdS2E);
	void incrementStateIdAmba(StateIdS2E);

	const std::string m_name;
	ControlFlowGraph *const m_cfg;

	u64 state_count = 0;
	std::unordered_map<StateIdS2E, StateIdAmba> m_states {};
	std::vector<NodeMetadataFFIPair> m_new_edges {};
};

} // namespace control_flow
