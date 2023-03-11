#pragma once

#include <unordered_map>
#include <unordered_set>

#include "S2EForwardDeclarations.h"
#include "Numbers.h"

namespace control_flow {

using BlockId = u64;

struct Block {
	BlockId id;
	std::unordered_set<BlockId> from;
	std::unordered_set<BlockId> to;
};

class ControlFlow {
  protected:
	std::unordered_map<u64, Block> m_cfg;
	BlockId last;

  public:
	ControlFlow () {}

	void onBlockStart(
		s2e::S2EExecutionState *s2e_state,
		u64 pc
	);
};

}
