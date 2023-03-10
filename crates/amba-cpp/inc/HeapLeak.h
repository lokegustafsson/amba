#pragma once

#include <s2e/S2EExecutionState.h>

#include <vector>

#include "Amba.h"

namespace heap_leak {
class HeapLeak {
  protected:
	std::vector<amba::AddressLengthPair> m_allocations;

  public:
	HeapLeak() {}

	void onMalloc(s2e::S2EExecutionState *state, u64 pc);

	void onFree(s2e::S2EExecutionState *state, u64 pc);

	void derefLeakCheck(s2e::S2EExecutionState *state, u64 pc);
};
} // namespace heap_leak
