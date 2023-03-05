// 3rd party library headers
#include <s2e/ConfigFile.h>
#include <s2e/S2E.h>
#include <s2e/Utils.h>
#include <s2e/S2EDeviceState.h>
#include <klee/Expr.h>
#include <Zydis/SharedTypes.h>
// #include <cpu/types.h>

// Standard library headers
#include <algorithm>

// Our headers
#include "Amba.h"
#include "AmbaException.h"
#include "Numbers.h"
#include "Zydis.h"

static const zydis::Decoder DECODER;

#define SPAN(d) \
	(std::span{d.data(), d.size()})

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(Amba, "Amba S2E plugin", "", );
#define SUBSCRIBE(fn) signal->connect(sigc::mem_fun(*this, (fn)));

void Amba::initialize() {
	auto& s2e = *this->s2e();
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
		this->getDebugStream(state) << "Translating block at " << hexval(pc) << "\n";
	}
	if (this->m_traceBlockExecution) {
		SUBSCRIBE(&Amba::slotExecuteBlockStart);
	}
}

void Amba::slotExecuteBlockStart(S2EExecutionState *state, u64 pc) {
	getDebugStream(state) << "Executing block at " << hexval(pc) << "\n";
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
		// Add return value + alloc size to this->m_allocations
}

void Amba::onDeref(S2EExecutionState *state, u64 pc) {
	// Check if read adr is on stack or within saved heap data
}

} // namespace plugins
} // namespace s2e

namespace amba {

// Read one instruction's worth of memory. Will throw if read memory
// is symbolic rather than constant
// (x86 has a limit of 15 bytes per instruction)
std::array<u8, MAX_INSTRUCTION_LENGTH> readConstantMemory(s2e::S2EExecutionState *state, u64 pc) {
	// TODO: Investigate if this actually works on both big and
	// low endian systems

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
