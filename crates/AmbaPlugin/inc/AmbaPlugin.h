#pragma once

#include <s2e/S2EExecutionState.h>
#include <klee/Searcher.h>

#include <memory>
#include <atomic>
#include <thread>
#include <mutex>

#include "Amba.h"
#include "HeapLeak.h"
#include "Numbers.h"
#include "AssemblyGraph.h"
#include "SymbolicGraph.h"
#include "LibambaRs.h"

namespace s2e {
namespace plugins {

class AmbaPlugin : public Plugin {
	S2E_PLUGIN
  public:
	explicit AmbaPlugin(S2E *s2e);
	~AmbaPlugin();

	void initialize();

	amba::TranslationFunction translateInstructionStart;
	amba::TranslationFunction translateBlockStart;
	amba::TranslationCompleteFunction translateBlockComplete;
	amba::ModuleFunction onModuleLoad;
	amba::ModuleFunction onModuleUnload;
	amba::ProcessFunction onProcessUnload;
	amba::TimerFunction onTimer;
	amba::TimerFunction onEngineShutdown;
	amba::StateKillFunction onStateKill;
	amba::StateMergeFunction onStateSwitch;

  protected:
	Ipc *const m_ipc;
	ModuleMap *m_modules = nullptr;
	std::string m_module_path = "";
	u64 m_module_pid = 0;
	std::atomic<bool> m_alive = true;
	std::atomic<klee::Searcher *> m_next_searcher = nullptr;

	std::mutex m_dead_states_lock;
	std::unordered_set<i32> m_dead_states;
	std::jthread m_ipc_receiver_thread;
	heap_leak::HeapLeak m_heap_leak;
	assembly_graph::AssemblyGraph m_assembly_graph;
	symbolic_graph::SymbolicGraph m_symbolic_graph;
};

} // namespace plugins
} // namespace s2e
