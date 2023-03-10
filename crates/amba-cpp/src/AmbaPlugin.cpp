// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>

// Our headers
#include "AmbaPlugin.h"
#include "AmbaData.h"
#include "HeapLeak.h"

#define SUBSCRIBE(fn) \
	(signal->connect(sigc::mem_fun(*this, (fn))))

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", );

AmbaPlugin::AmbaPlugin(S2E *s2e)
		: Plugin(s2e)
		, m_amba_data(std::make_unique<data::AmbaData>(data::AmbaData()))
	{}

void AmbaPlugin::initialize() {
	auto& core = *this->s2e()->getCorePlugin();

	// Set up event callbacks
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateInstructionStart
		));
}

void AmbaPlugin::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	[[maybe_unused]] TranslationBlock *tb,
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
	this->m_amba_data->m_heap_leak->onMalloc(state, pc);
}

void AmbaPlugin::onFree(S2EExecutionState *state, u64 pc) {
	this->m_amba_data->m_heap_leak->onFree(state, pc);
}

void AmbaPlugin::onDeref(S2EExecutionState *state, u64 pc) {
	this->m_amba_data->m_heap_leak->derefLeakCheck(state, pc);
}
} // namespace plugins
} // namespace s2e
