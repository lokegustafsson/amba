#pragma once

#include <s2e/Plugin.h>

#include <memory>
#include <unordered_map>

#include "S2EForwardDeclarations.h"
#include "Numbers.h"
#include "HeapLeak.h"

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
	TranslationFunction translateBlockStart;

  protected:
	heap_leak::HeapLeak m_heap_leak;
	std::unique_ptr<data::AmbaData> m_amba_data;
	std::unordered_map<u64, control_flow::Block> m_cfg;
	control_flow::ControlFlowState m_cfg_state;
};

} // namespace plugins
} // namespace s2e
