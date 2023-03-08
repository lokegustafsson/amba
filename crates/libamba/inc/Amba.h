#pragma once

#include <cpu/types.h>
#include <s2e/S2EExecutionState.h>

#include <compare>
#include <optional>

#include "Numbers.h"
#include "Zydis.h"

namespace amba {

using TranslationFunction = void (
	s2e::ExecutionSignal *,
	s2e::S2EExecutionState *state,
	TranslationBlock *tb,
	u64 p
);
using ExecutionFunction = void (s2e::S2EExecutionState *state, u64 pc);

struct AddressLengthPair {
	target_phys_addr_t adr;
	target_phys_addr_t size;

	auto operator<=>(const AddressLengthPair &rhs) const = default;
	bool operator==(const AddressLengthPair &rhs) const = default;
};

constexpr size_t MAX_INSTRUCTION_LENGTH = 15; // bytes

std::optional<target_phys_addr_t> readOperandAddress(
	const CPUX86State &cpu_state,
	const ZydisDecodedOperand operand
);

std::array<uint8_t, MAX_INSTRUCTION_LENGTH> readConstantMemory(S2EExecutionState *state, uint64_t pc);

zydis::Instruction readInstruction(s2e::S2EExecutionState *state, u64 pc);

bool isStackAddress(const CPUX86State &state, target_phys_addr_t adr);

u64 readRegister(const CPUX86State &state, const ZydisRegister reg);

void printBlock(
	const s2e::ExecutionSignal *const signal,
	const s2e::S2EExecutionState *const state,
	const TranslationBlock *const tb,
	const u64 pc
);

} // namespace amba
