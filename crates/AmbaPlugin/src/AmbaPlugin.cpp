// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>
#include <s2e/Plugins/OSMonitors/Support/ModuleMap.h>
#include <s2e/Plugins/OSMonitors/OSMonitor.h>

#include <memory.h>
#include <thread>

// Our headers
#include "AmbaPlugin.h"
#include "Amba.h"
#include "StatePrioritisation.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", "ModuleMap", "OSMonitor");

AmbaPlugin::AmbaPlugin(S2E *s2e)
	: Plugin(s2e)
	, m_ipc(rust_new_ipc())
	, m_ipc_receiver_thread(std::jthread {})
	, m_heap_leak(heap_leak::HeapLeak {})
	, m_assembly_graph(assembly_graph::AssemblyGraph { "basic blocks", s2e->getPlugin<ModuleMap>() })
	, m_symbolic_graph(symbolic_graph::SymbolicGraph { "symbolic states" })
{
	auto self = this;
	amba::debug_stream = [=](){ return &self->getDebugStream(); };
	amba::info_stream = [=](){ return &self->getInfoStream(); };
	amba::warning_stream = [=](){ return &self->getWarningsStream(); };
}

AmbaPlugin::~AmbaPlugin() {
	this->m_alive = false;
	this->m_ipc_receiver_thread.join();
}

void AmbaPlugin::initialize() {
	*amba::debug_stream() << "Begin initializing AmbaPlugin\n";

	auto s2e = this->s2e();
	auto& core = *s2e->getCorePlugin();
	this->m_modules = s2e->getPlugin<ModuleMap>();
	OSMonitor *monitor = static_cast<OSMonitor *>(s2e->getPlugin("OSMonitor"));

	bool ok;
	this->m_module_path = s2e
		->getConfig()
		->getString(
			this->getConfigKey() + ".module_path",
			"",
			&ok
		);
	if (!ok || this->m_module_path.empty()) {
		*amba::debug_stream()
			<< "NO `module_path` PROVIDED IN THE LUA CONFIG! "
			<< "Cannot continue.\n";
		return;
	}
	*amba::debug_stream()
		<< "Using module_path: "
		<< this->m_module_path
		<< '\n';

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
	core.onTranslateBlockComplete
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::translateBlockComplete
		));
	core.onStateFork
		.connect(sigc::mem_fun(
			this->m_assembly_graph,
			&assembly_graph::AssemblyGraph::onStateFork
		));
	core.onStateFork
		.connect(sigc::mem_fun(
			this->m_symbolic_graph,
			&symbolic_graph::SymbolicGraph::onStateFork
		));
	core.onStateMerge
		.connect(sigc::mem_fun(
			this->m_assembly_graph,
			&assembly_graph::AssemblyGraph::onStateMerge
		));
	core.onStateMerge
		.connect(sigc::mem_fun(
			this->m_symbolic_graph,
			&symbolic_graph::SymbolicGraph::onStateMerge
		));
	core.onTimer
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onTimer
		));
	core.onEngineShutdown
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onEngineShutdown
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

	auto self = this;
	this->m_ipc_receiver_thread = std::jthread([=]() {
		state_prioritisation::ipcReceiver(self->m_ipc, &self->m_alive, self->s2e());
	});
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
	auto mod = this->m_modules->getModule(state);
	if (mod.get() == nullptr || mod->Path != this->m_module_path) {
		return;
	}

	signal->connect(sigc::mem_fun(
		this->m_assembly_graph,
		&assembly_graph::AssemblyGraph::onBlockStart
	));
}
void AmbaPlugin::translateBlockComplete(
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 final_instruction_pc
) {
	auto mod = this->m_modules->getModule(state);
	if (mod.get() == nullptr || mod->Path != this->m_module_path) {
		return;
	}

	this->m_assembly_graph.translateBlockComplete(state, tb, final_instruction_pc);
}

void AmbaPlugin::onModuleLoad(
	S2EExecutionState *state,
	const ModuleDescriptor &module
) {
	if (module.Path != this->m_module_path) {
		return;
	}

	this->m_module_pid = module.Pid;
}

void AmbaPlugin::onModuleUnload(
	S2EExecutionState *state,
	const ModuleDescriptor &module
) {
	if (module.Path == this->m_module_path) {
		this->m_module_pid = 0;
	}
}

void AmbaPlugin::onProcessUnload(
	S2EExecutionState *state,
	const u64 cr3,
	const u64 pid,
	const u64 return_code
) {
	if (pid != this->m_module_pid) {
		return;
	}

	this->m_module_pid = 0;
	*amba::debug_stream()
		<< "Module "
		<< this->m_module_path
		<< " exited with code "
		<< std::to_string(return_code)
		<< '\n';
}

void AmbaPlugin::onTimer() {
	auto &symbolic_edges = this->m_symbolic_graph.edges();
	auto &assembly_edges = this->m_assembly_graph.edges();
	rust_ipc_send_edges(
		this->m_ipc,
		symbolic_edges.data(),
		(u64) symbolic_edges.size(),
		assembly_edges.data(),
		(u64) assembly_edges.size()
	);
	symbolic_edges.clear();
	assembly_edges.clear();

}

void AmbaPlugin::onEngineShutdown() {
	this->onTimer();
}

} // namespace plugins
} // namespace s2e
