// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>

// Our headers
#include "AmbaPlugin.h"
#include "AmbaData.h"
#include "HeapLeak.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", );

AmbaPlugin::AmbaPlugin(S2E *s2e)
	: Plugin(s2e)
	, m_amba_data(std::make_unique<data::AmbaData>(
			(data::AmbaData) {
				.heap_leak = heap_leak::HeapLeak()
			}
		))
	{}

void AmbaPlugin::initialize() {
	auto& debug = getDebugStream();
	debug << "Begin initializing AmbaPlugin\n";

	auto& core = *this->s2e()->getCorePlugin();

	// Set up event callbacks
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateInstructionStart
		));

	debug << "Finished initializing AmbaPlugin\n";
}

void AmbaPlugin::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	[[maybe_unused]] TranslationBlock *tb,
	u64 pc
) {
	const auto inst = amba::readInstruction(state, pc);

	if (inst.isCall()) {
		signal->connect(sigc::mem_fun(
			this->m_amba_data->heap_leak,
			&heap_leak::HeapLeak::onMalloc
		));
		signal->connect(sigc::mem_fun(
			this->m_amba_data->heap_leak,
			&heap_leak::HeapLeak::onFree
		));
	}
	if (inst.isDeref()) {
		signal->connect(sigc::mem_fun(
			this->m_amba_data->heap_leak,
			&heap_leak::HeapLeak::derefLeakCheck
		));
	}
}

} // namespace plugins
} // namespace s2e
