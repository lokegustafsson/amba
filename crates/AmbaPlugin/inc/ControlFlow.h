#pragma once

#include <s2e/S2EExecutionState.h>

#include <unordered_map>

#include "Numbers.h"
#include "Amba.h"
#include "LibambaRs.h"
#include "HashableWrapper.h"

namespace control_flow {

namespace types {

using StateIdS2E = hashable_wrapper::HashableWrapper<i32, 0>;
using IdAmba = hashable_wrapper::HashableWrapper<u64, 1>;
using StatePC = hashable_wrapper::HashableWrapper<u64, 2>;
using BasicBlockGeneration = hashable_wrapper::HashableWrapper<u8, 3>;
using PackedNodeData = hashable_wrapper::HashableWrapper<u64, 4>;

struct Unpacked {
	u64 vaddr;
	u8 gen;
	u64 state;
};

}

using namespace types;

StateIdS2E getStateIdS2E(s2e::S2EExecutionState *);

class ControlFlow {
  public:
	ControlFlow(std::string);
	~ControlFlow();

	const char *getName() const;
	ControlFlowGraph *cfg();

  protected:
	IdAmba getIdAmba(StateIdS2E);
	void incrementIdAmba(StateIdS2E);

	const std::string m_name;
	ControlFlowGraph *const m_cfg;

	u64 next_id = 0;
	std::unordered_map<StateIdS2E, IdAmba> m_states {};
};

} // namespace control_flow
