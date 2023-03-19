// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>

// Our headers
#include "Amba.h"
#include "AmbaPlugin.h"
#include "ControlFlow.h"
#include "HeapLeak.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", );

AmbaPlugin::AmbaPlugin(S2E *s2e)
	: Plugin(s2e)
	, m_heap_leak(heap_leak::HeapLeak {})
	, m_assembly_graph(control_flow::ControlFlow {})
	, m_symbolic_graph(control_flow::ControlFlow {})
{
	auto self = this;
	amba::debug_stream = [=](){ return &self->getDebugStream(); };
	amba::info_stream = [=](){ return &self->getInfoStream(); };
	amba::warning_stream = [=](){ return &self->getWarningsStream(); };
}

void AmbaPlugin::initialize() {
	*amba::debug_stream() << "Begin initializing AmbaPlugin\n";

	auto& core = *this->s2e()->getCorePlugin();

	// Set up event callbacks
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateInstructionStart
		));
	core.onTranslateBlockStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateBlockStart
		));
	core.onStateFork
		.connect(sigc::mem_fun(
			this->m_symbolic_graph,
			&control_flow::ControlFlow::onStateFork
		));

	*amba::debug_stream() << "Finished initializing AmbaPlugin\n";
}

void AmbaPlugin::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	//*amba::debug_stream() << "Translating instruction at " << hexval(pc) << '\n';

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

void AmbaPlugin::translateBlockStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	signal->connect(sigc::mem_fun(
		this->m_assembly_graph,
		&control_flow::ControlFlow::onBlockStart
	));
}

} // namespace plugins
} // namespace s2e
