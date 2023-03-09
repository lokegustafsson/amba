#pragma once

#include <s2e/Plugin.h>
#include <s2e/S2EExecutionState.h>

#include <vector>
#include <memory>

#include "Amba.h"
#include "ControlFlow.h"
#include "HeapLeak.h"
#include "Numbers.h"

namespace s2e {
namespace plugins {

class AmbaPlugin : public Plugin {
	S2E_PLUGIN
  public:
	explicit AmbaPlugin(S2E *s2e)
		: Plugin(s2e)
		, m_control_flow(std::make_unique<control_flow::ControlFlow>(control_flow::ControlFlow()))
		, m_heap_leak(std::make_unique<heap_leak::HeapLeak>(heap_leak::HeapLeak()))
	{}

	using TranslationFunction = void (
		s2e::ExecutionSignal *,
		s2e::S2EExecutionState *state,
		TranslationBlock *tb,
		u64 p
	);
	using ExecutionFunction = void (s2e::S2EExecutionState *state, u64 pc);

	void initialize();

	TranslationFunction translateInstructionStart;
	TranslationFunction translateBlockStart;
	ExecutionFunction onMalloc;
	ExecutionFunction onFree;
	ExecutionFunction onDeref;
	ExecutionFunction onBlockStart;

  protected:
	std::unique_ptr<control_flow::ControlFlow> m_control_flow;
	std::unique_ptr<heap_leak::HeapLeak> m_heap_leak;
};

} // namespace plugins
} // namespace s2e
