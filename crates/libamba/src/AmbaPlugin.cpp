// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>

// Our headers
#include "AmbaPlugin.h"
#include "HeapLeak.h"
#include "ControlFlow.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", );

AmbaPlugin::AmbaPlugin(S2E *s2e)
	: Plugin(s2e)
	, m_heap_leak(heap_leak::HeapLeak {})
	, m_amba_data(std::make_unique<data::AmbaData>(
			(data::AmbaData) {
				.heap_leak = heap_leak::HeapLeak(),
				.control_flow = control_flow::ControlFlow(),
			}
		))
	{}

void AmbaPlugin::initialize() {
	auto& debug = this->getDebugStream();
	debug << "Begin initializing AmbaPlugin\n";

	auto& core = *this->s2e()->getCorePlugin();

	// Set up event callbacks
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateInstructionStart
		));
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateBlockStart
		));

	debug << "Finished initializing AmbaPlugin\n";
}

void AmbaPlugin::translateBlockStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	signal->connect(sigc::mem_fun(
		this->m_amba_data->control_flow,
		&control_flow::ControlFlow::onBlockStart
	));
}

void AmbaPlugin::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	auto& debug = this->getDebugStream();
	debug << "Translating instruction at " << hexval(pc) << '\n';

	/*
	const auto inst = amba::readInstruction(state, pc);
	if (inst.isCall()) {
		signal->connect(sigc::mem_fun(
			this->m_heap_leak,
			&heap_leak::HeapLeak::onMalloc
		));
		signal->connect(sigc::mem_fun(
			this->m_heap_leak,
			&heap_leak::HeapLeak::onFree
		));
	}
	if (inst.isDeref()) {
		signal->connect(sigc::mem_fun(
			this->m_heap_leak,
			&heap_leak::HeapLeak::derefLeakCheck
		));
	}
	*/
}

} // namespace plugins
} // namespace s2e
