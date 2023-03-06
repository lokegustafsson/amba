// 3rd party library headers
#include <s2e/ConfigFile.h>
#include <s2e/S2E.h>
#include <s2e/Utils.h>
#include <s2e/S2EDeviceState.h>
#include <s2e/S2EExecutionStateRegisters.h>
#include <klee/Expr.h>
#include <Zydis/SharedTypes.h>
#include <Zydis/DecoderTypes.h>
#include <cpu/i386/cpu.h>
#include <cpu/types.h>

// Standard library
#include <algorithm>

// Our headers
#include "Amba.h"
#include "AmbaException.h"
#include "Numbers.h"
#include "Zydis.h"

#define SPAN(d) \
	(std::span{ d.data(), d.size() })
#define SUBSCRIBE(fn) \
	(signal->connect(sigc::mem_fun(*this, (fn))))

static const zydis::Decoder DECODER;

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(Amba, "Amba S2E plugin", "", );

void Amba::initialize() {
	const auto& s2e = *this->s2e();
	auto& core = *s2e.getCorePlugin();
	auto& config = *s2e.getConfig();
	const auto& key = this->getConfigKey();

	// Copying values from lua config?
	this->m_traceBlockTranslation
		= config.getBool(key + ".traceBlockTranslation");
	this->m_traceBlockExecution
		= config.getBool(key + ".traceBlockExecution");

	// Set up event callbacks
	core.onTranslateBlockStart
		.connect(sigc::mem_fun(
			*this,
			&Amba::slotTranslateBlockStart
		));
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&Amba::translateInstructionStart
		));
}

void Amba::slotTranslateBlockStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	if (this->m_traceBlockTranslation) {
		this->getDebugStream(state)
			<< "Translating block at "
			<< hexval{pc}
			<< "\n";
	}
	if (this->m_traceBlockExecution) {
		SUBSCRIBE(&Amba::slotExecuteBlockStart);
	}
}

void Amba::slotExecuteBlockStart(S2EExecutionState *state, u64 pc) {
	this->getDebugStream(state)
		<< "Executing block at "
		<< hexval{pc}
		<< "\n";
}

void Amba::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	const auto inst = amba::readInstruction(state, pc);

	if (inst.isCall()) {
		SUBSCRIBE(&Amba::onFunctionCall);
	}
	if (inst.isDeref()) {
		SUBSCRIBE(&Amba::onDeref);
	}
}

void Amba::onFunctionCall(S2EExecutionState *state, u64 pc) {
	// if malloc
		// Add return value + alloc size to this->m_allocations (make sure it's sorted)
	// if free
		// Remove from m_allocations
	const auto inst = amba::readInstruction(state, pc);

	AMBA_ASSERT(inst.m_inst.mnemonic != ZYDIS_MNEMONIC_CALL);
	AMBA_ASSERT(inst.m_ops.size() != 1);
}

void Amba::onDeref(S2EExecutionState *state, u64 pc) {
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

} // namespace plugins
} // namespace s2e

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
