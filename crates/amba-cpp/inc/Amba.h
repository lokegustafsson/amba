#pragma once

#include <cpu/types.h>
#include <s2e/CorePlugin.h>
#include <s2e/Plugin.h>
#include <s2e/S2EExecutionState.h>

#include <unordered_map>

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
	TranslationFunction translateInstructionStart;
	ExecutionFunction slotExecuteBlockStart;
	ExecutionFunction onFunctionCall;
	ExecutionFunction onDeref;

  protected:
	bool m_traceBlockTranslation;
	bool m_traceBlockExecution;

	std::unordered_map<target_phys_addr_t, target_phys_addr_t> m_allocations;
};

} // namespace plugins
} // namespace s2e
