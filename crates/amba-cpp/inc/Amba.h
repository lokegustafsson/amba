#pragma once

#include <s2e/CorePlugin.h>
#include <s2e/Plugin.h>
#include <s2e/S2EExecutionState.h>

namespace s2e {
namespace plugins {

class Amba : public Plugin {
	S2E_PLUGIN
public:
	Amba(S2E *s2e) : Plugin(s2e) {}

	using TranslationFunction = void (
		s2e::ExecutionSignal *,
		s2e::S2EExecutionState *state,
		TranslationBlock *tb,
		uint64_t p
	);
	using ExecutionFunction = void (s2e::S2EExecutionState *state, uint64_t pc);

	void initialize();

	TranslationFunction slotTranslateBlockStart;
	ExecutionFunction slotExecuteBlockStart;

private:
	bool m_traceBlockTranslation;
	bool m_traceBlockExecution;
};

} // namespace plugins
} // namespace s2e
