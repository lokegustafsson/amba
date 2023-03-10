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

	amba::ExecutionFunction onMalloc;

	amba::ExecutionFunction onFree;

	amba::ExecutionFunction derefLeakCheck;
};
} // namespace heap_leak
