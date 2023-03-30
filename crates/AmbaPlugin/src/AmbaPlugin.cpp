// 3rd party library headers
#include <s2e/S2E.h>
#include <s2e/Utils.h>
#include <s2e/Plugins/OSMonitors/Support/ModuleMap.h>
#include <s2e/Plugins/OSMonitors/OSMonitor.h>

// Our headers
#include "Amba.h"
#include "AmbaPlugin.h"
#include "HeapLeak.h"
#include "LibambaRs.h"

namespace s2e {
namespace plugins {

S2E_DEFINE_PLUGIN(AmbaPlugin, "Amba S2E plugin", "", "ModuleMap", "OSMonitor");

AmbaPlugin::AmbaPlugin(S2E *s2e)
	: Plugin(s2e)
	, m_heap_leak(heap_leak::HeapLeak {})
{
	auto self = this;
	amba::debug_stream = [=](){ return &self->getDebugStream(); };
	amba::info_stream = [=](){ return &self->getInfoStream(); };
	amba::warning_stream = [=](){ return &self->getWarningsStream(); };
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

	// Rust hooks
	core.onEngineShutdown
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onEngineShutdown
		));
	core.onStateFork
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onStateFork
		));
	core.onStateMerge
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onStateMerge
		));
	core.onTranslateBlockStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onTranslateBlockStart
		));

	// Module bookkeeping
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

	// Inactivated and paused heap overflow detector
	core.onTranslateInstructionStart
		.connect(sigc::mem_fun(
			*this,
			&AmbaPlugin::onTranslateInstructionStart
		));

	(void) core.onStateForkDecide;
	(void) core.onStateKill;
	(void) core.onStateSwitch;

	rust_init();

	*amba::debug_stream() << "Finished initializing AmbaPlugin\n";
}

void AmbaPlugin::onEngineShutdown() {
	rust_on_engine_shutdown();
}

void AmbaPlugin::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	std::vector<u32> new_state_ids {};
	for (auto &new_state : new_states) {
		new_state_ids.push_back(new_state->getID());
	}
	rust_on_state_fork(old_state->getID(), new_state_ids.data(), new_state_ids.size());
}

void AmbaPlugin::onStateMerge(
	s2e::S2EExecutionState *base_state,
	s2e::S2EExecutionState *other_state
) {
	rust_on_state_merge(base_state->getID(), other_state->getID());
}

void AmbaPlugin::onTranslateBlockStart(
	ExecutionSignal *signal,
	S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	if (!this->m_module_pid) {
		return;
	}

	auto mod = this->m_modules->getModule(state);
	u64 module_internal_offset = 0;
	if (mod) {
		mod->ToNativeBase(pc, module_internal_offset);
	}
	const char* module_path_cstr = mod ? mod->Path.c_str() : nullptr;
	rust_on_translate_block(pc, nullptr, 0, module_path_cstr, module_internal_offset);

	signal->connect(sigc::mem_fun(
		*this,
		&AmbaPlugin::onBlockStart
	));
}

void AmbaPlugin::onModuleLoad(
	S2EExecutionState *state,
	const ModuleDescriptor &module
) {
	if (module.Path != this->m_module_path) {
		return;
	}

	this->m_module_pid = module.Pid;
	*amba::debug_stream() << "Loaded module " << this->m_module_path << '\n';
	for (const auto& section : module.Sections) {
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

void AmbaPlugin::onBlockStart(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	rust_on_watched_block_start_execute(pc);
}

void AmbaPlugin::onTranslateInstructionStart(
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


} // namespace plugins
} // namespace s2e
