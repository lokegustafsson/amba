#pragma once

#include <s2e/Plugin.h>
#include <s2e/S2EExecutionState.h>

#include "HeapLeak.h"
#include "Numbers.h"
#include "ControlFlow.h"

namespace s2e {
namespace plugins {

class AmbaPlugin : public Plugin {
	S2E_PLUGIN
  public:
	explicit AmbaPlugin(S2E *s2e);

	using TranslationFunction = void (
		s2e::ExecutionSignal *,
		s2e::S2EExecutionState *state,
		TranslationBlock *tb,
		u64 p
	);
	using ExecutionFunction = void (s2e::S2EExecutionState *state, u64 pc);

	void initialize();

	TranslationFunction translateInstructionStart;

  protected:
	heap_leak::HeapLeak m_heap_leak;
	control_flow::ControlFlow m_control_flow;
};

} // namespace plugins
} // namespace s2e
