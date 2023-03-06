// 3rd party library headers
#include <s2e/ConfigFile.h>
#include <s2e/S2E.h>
#include <s2e/Utils.h>
#include <s2e/S2EDeviceState.h>
#include <s2e/S2EExecutionStateRegisters.h>
#include <klee/Expr.h>
#include <Zydis/SharedTypes.h>
#include <Zydis/DecoderTypes.h>

// Standard library
#include <algorithm>

// Our headers
#include "Amba.h"
#include "AmbaPlugin.h"
#include "AmbaException.h"
#include "HeapLeak.h"
#include "Numbers.h"
#include "Zydis.h"

#define SUBSCRIBE(fn) \
	(signal->connect(sigc::mem_fun(*this, (fn))))

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", );

void AmbaPlugin::initialize() {
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
			&AmbaPlugin::slotTranslateBlockStart
		));
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateInstructionStart
		));
}

void AmbaPlugin::slotTranslateBlockStart(
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
		SUBSCRIBE(&AmbaPlugin::slotExecuteBlockStart);
	}
}

void AmbaPlugin::slotExecuteBlockStart(S2EExecutionState *state, u64 pc) {
	this->getDebugStream(state)
		<< "Executing block at "
		<< hexval{pc}
		<< "\n";
}

void AmbaPlugin::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	const auto inst = amba::readInstruction(state, pc);

	if (inst.isCall()) {
		// SUBSCRIBE(&AmbaPlugin::onMalloc);
		// SUBSCRIBE(&AmbaPlugin::onFree);
	}
	if (inst.isDeref()) {
		SUBSCRIBE(&AmbaPlugin::onDeref);
	}
}

void AmbaPlugin::onMalloc(S2EExecutionState *state, u64 pc) {
	heap_leak::onMalloc(state, pc, &this->m_allocations);
}

void AmbaPlugin::onFree(S2EExecutionState *state, u64 pc) {
	heap_leak::onFree(state, pc, &this->m_allocations);
}

void AmbaPlugin::onDeref(S2EExecutionState *state, u64 pc) {
	heap_leak::derefLeakCheck(state, pc, &this->m_allocations);
}

} // namespace plugins
} // namespace s2e
