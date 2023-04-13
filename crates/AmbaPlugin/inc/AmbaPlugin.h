#pragma once

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

	void initialize();

	amba::TranslationFunction translateInstructionStart;
	amba::TranslationFunction translateBlockStart;
	amba::ModuleFunction onModuleLoad;
	amba::ModuleFunction onModuleUnload;
	amba::ProcessFunction onProcessUnload;

  protected:
	ModuleMap *m_modules = nullptr;
	std::string m_module_path = "";
	u64 m_module_pid = 0;

	heap_leak::HeapLeak m_heap_leak;
	control_flow::ControlFlow m_assembly_graph;
	control_flow::ControlFlow m_symbolic_graph;
};

} // namespace plugins
} // namespace s2e
