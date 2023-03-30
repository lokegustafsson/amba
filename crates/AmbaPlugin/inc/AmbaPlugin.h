#pragma once

#include <s2e/S2EExecutionState.h>

#include "HeapLeak.h"
#include "Numbers.h"

class ModuleMap;
class ModuleDescriptor;

namespace s2e {
namespace plugins {

class AmbaPlugin : public Plugin {
	S2E_PLUGIN
  public:
	explicit AmbaPlugin(S2E *s2e);

	using EngineShutdownFunction = void ();
	using StateForkFunction = void (
		s2e::S2EExecutionState *state,
		const std::vector<s2e::S2EExecutionState *> &,
		const std::vector<klee::ref<klee::Expr>> &
	);
	using StateMergeFunction = void (
		s2e::S2EExecutionState *dest,
		s2e::S2EExecutionState *source
	);
	using TranslationFunction = void (
		s2e::ExecutionSignal *,
		s2e::S2EExecutionState *state,
		TranslationBlock *tb,
		u64 p
	);
	using ModuleFunction = void (S2EExecutionState *state, const ModuleDescriptor &module);
	using ProcessFunction = void (S2EExecutionState *state, const u64 cr3, const u64 pid, const u64 return_code);


	void initialize();

	EngineShutdownFunction onEngineShutdown;
	StateForkFunction onStateFork;
	StateMergeFunction onStateMerge;
	TranslationFunction onTranslateBlockStart;
	amba::ExecutionFunction onBlockStart;

	ModuleFunction onModuleLoad;
	ModuleFunction onModuleUnload;
	ProcessFunction onProcessUnload;

	TranslationFunction onTranslateInstructionStart;

  protected:
	ModuleMap *m_modules = nullptr;
	std::string m_module_path = "";
	u64 m_module_pid = 0;

	heap_leak::HeapLeak m_heap_leak;
};

} // namespace plugins
} // namespace s2e
