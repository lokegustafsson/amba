#pragma once

#include <cpu/types.h>
#include <s2e/CorePlugin.h>
#include <s2e/Plugin.h>
#include <s2e/S2EExecutionState.h>

#include <queue>

#include "Zydis.h"

namespace amba {

struct AddressLengthPair {
	target_phys_addr_t adr;
	target_phys_addr_t size;

	auto operator<=>(const AddressLengthPair &rhs) const = default;
	bool operator==(const AddressLengthPair &rhs) const = default;
};

constexpr size_t MAX_INSTRUCTION_LENGTH = 15; // bytes

std::array<uint8_t, MAX_INSTRUCTION_LENGTH> readConstantMemory(S2EExecutionState *state, uint64_t pc);
zydis::Instruction readInstruction(s2e::S2EExecutionState *state, u64 pc);
bool isStackAddress(void *adr);

} // namespace amba

namespace s2e {
namespace plugins {

class Amba : public Plugin {
	S2E_PLUGIN
  public:
	Amba(S2E *s2e) : Plugin(s2e) {}

	using TranslationFunction = void (
		s2e::ExecutionSignal *,
		s2e::S2EExecutionState *state,
		TranslationBlock *tb,
		uint64_t p
	);
	using ExecutionFunction = void (s2e::S2EExecutionState *state, uint64_t pc);

	void initialize();

	TranslationFunction slotTranslateBlockStart;
	TranslationFunction translateInstructionStart;
	ExecutionFunction slotExecuteBlockStart;
	ExecutionFunction onFunctionCall;
	ExecutionFunction onDeref;

  protected:
	bool m_traceBlockTranslation;
	bool m_traceBlockExecution;

	std::priority_queue<amba::AddressLengthPair> m_allocations;
};

} // namespace plugins
} // namespace s2e