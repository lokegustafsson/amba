#pragma once

#include <s2e/S2EExecutionState.h>

#include <unordered_map>

#include "Numbers.h"
#include "Amba.h"
#include "LibambaRs.h"
#include "HashableWrapper.h"

namespace control_flow {

namespace types {
// Values used as keys need to have I = 0 or else the default
// constructor is implicitly deleted?
using UidS2E = hashable_wrapper::HashableWrapper<i32, 0>;
using StatePC = hashable_wrapper::HashableWrapper<u64, 0>;

using AmbaUid = hashable_wrapper::HashableWrapper<u64, 1>;
using Generation = hashable_wrapper::HashableWrapper<u8, 2>;
using Packed = hashable_wrapper::HashableWrapper<u64, 3>;

struct Unpacked {
	u64 vaddr;
	u8 gen;
	u64 state;
};

}

using namespace types;

void updateControlFlowGraph(ControlFlowGraph *, Packed, Packed);
Unpacked unpack(Packed);
UidS2E getID(s2e::S2EExecutionState *);

class ControlFlow {
  public:
	ControlFlow(std::string);
	~ControlFlow();

	const char *getName() const;
	ControlFlowGraph *cfg();

  protected:
	StatePC toAlias(UidS2E, u64);
	Packed getPacked(s2e::S2EExecutionState *, u64);
	AmbaUid getAmbaId(UidS2E);
	void incrementAmbaId(UidS2E);

	const std::string m_name;
	ControlFlowGraph *const m_cfg;

	std::unordered_map<UidS2E, AmbaUid> m_states {};
	std::unordered_map<StatePC, Generation> m_generations {};
	std::unordered_map<AmbaUid, Packed> m_last {};
};



} // namespace control_flow
