#pragma once

#include <s2e/S2EExecutionState.h>

#include "HeapLeak.h"
#include "Numbers.h"
#include "ControlFlow.h"

class ModuleMap;
class ModuleDescriptor;

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
	using ModuleFunction = void (S2EExecutionState *state, const ModuleDescriptor &module);
	using ProcessFunction = void (S2EExecutionState *state, const u64 cr3, const u64 pid, const u64 return_code);

	void initialize();

	TranslationFunction translateInstructionStart;
	TranslationFunction translateBlockStart;
	ModuleFunction onModuleLoad;
	ModuleFunction onModuleUnload;
	ProcessFunction onProcessUnload;


  protected:
	ModuleMap *m_modules;
	std::string m_module_path = "";
	u64 m_module_pid = 0;

	heap_leak::HeapLeak m_heap_leak;
	control_flow::ControlFlow m_assembly_graph;
	control_flow::ControlFlow m_symbolic_graph;
};

} // namespace plugins
} // namespace s2e
