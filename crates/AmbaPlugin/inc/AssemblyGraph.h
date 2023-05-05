#pragma once

#include <s2e/Plugins/OSMonitors/Support/ModuleMap.h>

#include <string>
#include <vector>

#include "ControlFlow.h"

namespace assembly_graph {

using namespace control_flow::types;

struct TranslationBlockMetadata {
	BasicBlockGeneration generation;
	u64 elf_vaddr;
	// NOTE: These are a lot of allocations (and per-block-execute copying).
	// Removing these is low-hanging fruit for later.
	std::vector<u8> content;
};

class AssemblyGraph : public control_flow::ControlFlow {
  public:
	AssemblyGraph(std::string name, s2e::plugins::ModuleMap *module_map);

	amba::TranslationFunction translateBlockStart;
	amba::ExecutionFunction onBlockStart;
	amba::SymbolicExecutionFunction onStateFork;
	amba::StateMergeFunction onStateMerge;

  protected:
	StatePC packStatePc(StateIdS2E, u64);
	BasicBlockMetadata getMetadata(s2e::S2EExecutionState *, u64);

	s2e::plugins::ModuleMap *m_module_map;
	std::unordered_map<StatePC, TranslationBlockMetadata> m_translation_block_metadata {};
	std::unordered_map<StateIdAmba, BasicBlockMetadata> m_last_executed_bb {};
};

}
