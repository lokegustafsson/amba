// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>
#include <s2e/Plugins/OSMonitors/Support/ModuleMap.h>
#include <s2e/Plugins/OSMonitors/OSMonitor.h>

// Our headers
#include "Amba.h"
#include "AmbaPlugin.h"
#include "ControlFlow.h"
#include "HeapLeak.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", "ModuleMap", "OSMonitor");

AmbaPlugin::AmbaPlugin(S2E *s2e)
	: Plugin(s2e)
	, m_module_pid(0)
	, m_heap_leak(heap_leak::HeapLeak {})
	, m_assembly_graph(control_flow::ControlFlow {})
	, m_symbolic_graph(control_flow::ControlFlow {})
{
	auto self = this;
	amba::debug_stream = [=](){ return &self->getDebugStream(); };
	amba::info_stream = [=](){ return &self->getInfoStream(); };
	amba::warning_stream = [=](){ return &self->getWarningsStream(); };
}

void AmbaPlugin::initialize() {
	*amba::debug_stream() << "Begin initializing AmbaPlugin\n";

	auto& core = *this->s2e()->getCorePlugin();
	m_modules = this->s2e()->getPlugin<ModuleMap>();
	OSMonitor *monitor = static_cast<OSMonitor *>(this->s2e()->getPlugin("OSMonitor"));

	// Set up event callbacks
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateInstructionStart
		));
	core.onTranslateBlockStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateBlockStart
		));
	core.onStateFork
		.connect(sigc::mem_fun(
			this->m_symbolic_graph,
			&control_flow::ControlFlow::onStateFork
		));
	core.onStateMerge
		.connect(sigc::mem_fun(
			this->m_symbolic_graph,
			&control_flow::ControlFlow::onStateMerge
		));

	monitor->onModuleLoad
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onModuleLoad
		));
	monitor->onModuleUnload
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onModuleUnload
		));
	monitor->onProcessUnload
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onProcessUnload
		));

	(void) core.onStateForkDecide;
	(void) core.onStateKill;
	(void) core.onStateSwitch;

	*amba::debug_stream() << "Finished initializing AmbaPlugin\n";
}

void AmbaPlugin::translateInstructionStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	//*amba::debug_stream() << "Translating instruction at " << hexval(pc) << '\n';

	/*
	const auto inst = amba::readInstruction(state, pc);
	if (inst.isCall()) {
		signal->connect(sigc::mem_fun(
			this->m_heap_leak,
			&heap_leak::HeapLeak::onMalloc
		));
		signal->connect(sigc::mem_fun(
			this->m_heap_leak,
			&heap_leak::HeapLeak::onFree
		));
	}
	if (inst.isDeref()) {
		signal->connect(sigc::mem_fun(
			this->m_heap_leak,
			&heap_leak::HeapLeak::derefLeakCheck
		));
	}
	*/
}

void AmbaPlugin::translateBlockStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	if (!this->m_module_pid) { return; }

	auto mod = this->m_modules->getModule(state);
	*amba::debug_stream()
		<< "Translating instruction at " << hexval(pc)
		<< (mod ? " in " + mod->Name : "")
		<< '\n';

	signal->connect(sigc::mem_fun(
		this->m_assembly_graph,
		&control_flow::ControlFlow::onBlockStart
	));
}

void AmbaPlugin::onModuleLoad(
	S2EExecutionState *state,
	const ModuleDescriptor &module
) {
	if (module.Path != "./hello") { return; }

	this->m_module_pid = module.Pid;
	*amba::debug_stream() << "Loaded our module\n";
	for (const auto& section: module.Sections) {
		*amba::debug_stream()
			<< "Found section (" << section.name << ")"
			<< " at " << hexval(section.nativeLoadBase)
			<< " with size " << std::to_string(section.size)
			<< '\n';
	};
}

void AmbaPlugin::onModuleUnload(
	S2EExecutionState *state,
	const ModuleDescriptor &module
) {
	if (module.Path == "./hello") {
		this->m_module_pid = 0;
	}
}

void AmbaPlugin::onProcessUnload(
	S2EExecutionState *state,
	uint64_t cr3,
	uint64_t pid,
	uint64_t return_code
) {
	if (pid != this->m_module_pid) { return; }

	this->m_module_pid = 0;
	*amba::debug_stream() << "Our module exited with code " << std::to_string(return_code) << '\n';
}

} // namespace plugins
} // namespace s2e
