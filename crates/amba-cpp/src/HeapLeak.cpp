#include <algorithm>

#include "HeapLeak.h"
#include "AmbaException.h"

namespace heap_leak {

void HeapLeak::onMalloc(s2e::S2EExecutionState *state, u64 pc) {}

void HeapLeak::onFree(s2e::S2EExecutionState *state, u64 pc) {}

void HeapLeak::derefLeakCheck(s2e::S2EExecutionState *state, u64 pc) {
	// Check if read adr is on stack or within saved heap data
	const auto operands = amba::readInstruction(state, pc).m_ops;
	const CPUX86State &cpu_state = *state->regs()->getCpuState();
	for (const auto& operand : operands) {
		// Skip operand if it doesn't contain a dereference
		const auto maybe_adr = amba::readOperandAddress(cpu_state, operand);
		if (!maybe_adr.has_value()) {
			continue;
		}
		const auto adr = *maybe_adr;

		if (!amba::isStackAddress(cpu_state, adr)) {
			// Loop through m_allocations, see if it's
			// within any allocation or if it's out of
			// bounds.

			const auto allocation = (amba::AddressLengthPair) {
				.adr = adr,
				.size = 0
			};

			// Get a pointer to the last allocation that is considered smaller (or equal?) to allocation
			const auto result = std::lower_bound( // std::upper_bound - 1?
				this->m_allocations.begin(),
				this->m_allocations.end(),
				allocation
			);

			if (result == this->m_allocations.end()) {
				continue;
			}

			// Assert that access is in bounds of the allocation
			AMBA_ASSERT(result->adr <= adr);
			AMBA_ASSERT(result->adr + result->size > adr);
		}
	}
}

}
