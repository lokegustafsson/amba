#include <algorithm>
#include <s2e/ConfigFile.h>
#include <s2e/S2E.h>
#include <s2e/Utils.h>

#include <cpu/types.h>

#include "Amba.h"
#include "Numbers.h"
#include "Zydis.h"
#include "Zydis/SharedTypes.h"

namespace s2e {
namespace plugins {

static const zydis::Decoder DECODER;

// Just a guess for now
constexpr size_t MAX_INSTRUCTION_LENGTH = 64; // bytes

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
	u8* memory;
	const auto inst = DECODER.decode(std::span {
		memory + pc,
		MAX_INSTRUCTION_LENGTH
	});

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
