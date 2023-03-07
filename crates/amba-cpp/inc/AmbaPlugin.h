#pragma once

#include <s2e/Plugin.h>
#include <s2e/S2EExecutionState.h>

#include <vector>

#include "Amba.h"
#include "Numbers.h"

namespace s2e {
namespace plugins {

class AmbaPlugin : public Plugin {
	S2E_PLUGIN
  public:
	explicit AmbaPlugin(S2E *s2e) : Plugin(s2e) {}

	using TranslationFunction = void (
		s2e::ExecutionSignal *,
		s2e::S2EExecutionState *state,
		TranslationBlock *tb,
		u64 p
	);
	using ExecutionFunction = void (s2e::S2EExecutionState *state, u64 pc);

	void initialize();

	TranslationFunction translateInstructionStart;
	ExecutionFunction onMalloc;
	ExecutionFunction onFree;
	ExecutionFunction onDeref;

  protected:
	std::vector<amba::AddressLengthPair> m_allocations;
};

} // namespace plugins
} // namespace s2e
