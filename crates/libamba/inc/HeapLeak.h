#pragma once

#include <vector>

#include "S2EForwardDeclarations.h"
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
