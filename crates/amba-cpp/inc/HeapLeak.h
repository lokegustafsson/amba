#pragma once

#include <s2e/S2EExecutionState.h>

#include <vector>

#include "Amba.h"

namespace heap_leak {

void onMalloc(
	s2e::S2EExecutionState *state,
	u64 pc,
	std::vector<amba::AddressLengthPair> *allocations
);

void onFree(
	s2e::S2EExecutionState *state,
	u64 pc,
	std::vector<amba::AddressLengthPair> *allocations
);

void derefLeakCheck(
	s2e::S2EExecutionState *state,
	u64 pc,
	std::vector<amba::AddressLengthPair> *allocations
);

}
