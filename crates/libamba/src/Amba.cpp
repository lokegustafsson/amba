#include <s2e/S2EExecutionState.h>
#include <cpu/i386/cpu.h>
#include <cpu/types.h>

#include "Amba.h"
#include "AmbaException.h"

#define SPAN(d) \
	(std::span{ d.data(), d.size() })

static const zydis::Decoder DECODER;

namespace amba {

// Get a pointer from an operand if it contains one (even through indexing operations)
std::optional<target_phys_addr_t> readOperandAddress(const CPUX86State &cpu_state, const ZydisDecodedOperand operand) {
	switch (operand.type) {
	case ZYDIS_OPERAND_TYPE_MEMORY: {
		const auto mem = operand.mem;
		// segment:displacement(base register, index register, scale factor)

		AMBA_ASSERT(!mem.segment); // Because who knows what this even is

		const i64 base = amba::readRegister(cpu_state, mem.base);
		const i64 index = amba::readRegister(cpu_state, mem.index);

		return (mem.disp.has_displacement ? mem.disp.value : 0)
			+ base + index * (i64) mem.scale;
	};

	case ZYDIS_OPERAND_TYPE_POINTER: {
		const auto ptr = operand.ptr;
		AMBA_ASSERT(!ptr.segment); // Because who knows what this even is

		return (i64) ptr.offset;
	};

	default: return std::nullopt;
	}
}

// Read one instruction's worth of memory. Will throw if read memory
// is symbolic rather than constant
// (x86 has a limit of 15 bytes per instruction)
std::array<u8, MAX_INSTRUCTION_LENGTH> readConstantMemory(s2e::S2EExecutionState *state, u64 pc) {
	// TODO: Investigate if this actually works on both big and
	// little endian systems

	// Is this repeating the same work over and over and over again?

	auto mem = state->mem();

	std::array<u8, MAX_INSTRUCTION_LENGTH> arr;
	for (size_t i = 0; i < MAX_INSTRUCTION_LENGTH; i++) {
		auto expr = mem->read(pc + i).get();
		if (expr->getKind() != klee::Expr::Constant) {
			AMBA_THROW();
		}
		// This is a major assumption, though confirmed by Loke
		arr[i] = ((klee::ConstantExpr *) expr)->getLimitedValue(0xFF);
	}
	return arr;
}

zydis::Instruction readInstruction(s2e::S2EExecutionState *state, u64 pc) {
	const auto mem = readConstantMemory(state, pc);
	return DECODER.decode(SPAN(mem));
}

bool isStackAddress(const CPUX86State &state, target_phys_addr_t adr) {
	const auto sp = state.regs[6];
	// https://stackoverflow.com/questions/1825964/c-c-maximum-stack-size-of-program-on-mainstream-oses
	constexpr target_phys_addr_t STACK_SIZE = 7.4 * prefixes::Mi;

	// Incorrect, but close enough.
	// We can't find the start address of the stack reasonably, so we just check within the stacksize of the current stack pointer
	// Doesn't handle leaf function frames either.
	// TODO: Check if this is calculating the stack in the correct direction
	return adr >= sp && adr <= sp + STACK_SIZE;
}

// Translate form ZydisRegister to S2E CPUX86State to read the value in a register
u64 readRegister(const CPUX86State &state, const ZydisRegister reg) {
	// TODO: Verify assumption that Zydis id and S2E ids line up

	const ZydisRegisterClass reg_class = ZydisRegisterGetClass(reg);
	const u8 reg_id = ZydisRegisterGetId(reg);

	AMBA_ASSERT(
		reg_class != ZYDIS_REGCLASS_INVALID
		&& reg_class != ZYDIS_REGCLASS_FLAGS
		&& reg_class != ZYDIS_REGCLASS_IP
	);

	switch (reg_class) {
	// General purpose registers in 8, 16, 32 and 64 bit varieties
	case ZYDIS_REGCLASS_GPR8:
	case ZYDIS_REGCLASS_GPR16:
	case ZYDIS_REGCLASS_GPR32:
	case ZYDIS_REGCLASS_GPR64: {
		return state.regs[reg_id];
	}
	// Classic floating point registers
	case ZYDIS_REGCLASS_X87: {
		return state.fpregs[reg_id].mmx.q;
	}
	// Modern floating point registers
	case ZYDIS_REGCLASS_MMX: {
		return state.fpregs[reg_id].mmx.q;
	}
	// SIMD registers
	case ZYDIS_REGCLASS_XMM:
	case ZYDIS_REGCLASS_YMM:
	case ZYDIS_REGCLASS_ZMM: {
		// Return the first 64 bits of the corresponding register
		// (Actual SIMD register is 128 bits)
		return state.xmm_regs[reg_id]._q[0];
	}
	default: {
		AMBA_THROW();
	}
	}
}

} // namespace amba
