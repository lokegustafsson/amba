#include <vector>
#include <s2e/Plugins/OSMonitors/ModuleDescriptor.h>
#include <s2e/Plugins/OSMonitors/Support/ModuleMap.h>
#include <tcg/tb.h>

#include "AssemblyGraph.h"
#include "AmbaException.h"
#include "ControlFlow.h"

namespace assembly_graph {

AssemblyGraph::AssemblyGraph(std::string name, s2e::plugins::ModuleMap *module_map)
	: ControlFlow(name)
	, m_module_map(module_map)
{}

void AssemblyGraph::translateBlockStart(
	s2e::ExecutionSignal *signal,
	s2e::S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	u64 elf_vaddr = 0;
	s2e::ModuleDescriptorConstPtr mod = this->m_module_map->getModule(state);
	if (mod != nullptr) {
		bool ok = mod->ToNativeBase(pc, elf_vaddr);
		assert(ok);
	}

	const u64 tb_vaddr = tb->pc;
	const u64 tb_len = tb->size;
	std::vector<u8> cached_tb_content(tb_len);
	bool ok = state->mem()->read(tb_vaddr, cached_tb_content.data(), tb_len);
	if (!ok) {
		// TODO: Write to the standard warning stream
		std::cerr << "TODO: Failed tb read tb_vaddr=" << tb_vaddr << " tb_len=" << tb_len << "\n";
		// TODO: This causes "silent concretizing". We should read memory in a way that
		// fails if it is symbolic. But on the other hand, how can a newly translated
		// TranslationBlock possibly be non-concrete?
	}

	const StatePC key = this->packStatePc(
		control_flow::getStateIdS2E(state),
		pc
	);
	++this->m_translation_block_metadata[key].generation.val;
	this->m_translation_block_metadata[key].elf_vaddr = elf_vaddr;
	this->m_translation_block_metadata[key].content = cached_tb_content;
}

void AssemblyGraph::onBlockStart(
	s2e::S2EExecutionState *state,
	u64 pc
) {
	const StateIdAmba amba_id = this->getStateIdAmba(control_flow::getStateIdS2E(state));
	const BasicBlockMetadata curr = this->getMetadata(state, pc);
	// Will insert 0 if value doesn't yet exist
	BasicBlockMetadata &last = this->m_last_executed_bb[amba_id];
	this->m_new_edges.push_back(
		(NodeMetadataFFIPair) {
			.fst = last.into_ffi(),
			.snd = curr.into_ffi()
		}
	);
	last = curr;
}

void AssemblyGraph::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	const StateIdAmba old_amba_id = this->getStateIdAmba(control_flow::getStateIdS2E(old_state));
	const BasicBlockMetadata old_last = this->m_last_executed_bb[old_amba_id];

	this->incrementStateIdAmba(control_flow::getStateIdS2E(old_state));

	for (auto& new_state : new_states) {
		const StateIdAmba new_amba_id = this->getStateIdAmba(control_flow::getStateIdS2E(new_state));
		this->m_last_executed_bb[new_amba_id] = old_last;
	}
}

void AssemblyGraph::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	this->incrementStateIdAmba(control_flow::getStateIdS2E(destination_state));
}

StatePC AssemblyGraph::packStatePc(StateIdS2E uid, u64 pc) {
	return pc << 4 | (u64) uid.val;
}

BasicBlockMetadata AssemblyGraph::getMetadata(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const StateIdS2E state = StateIdS2E(s2e_state->getID());
	const StateIdAmba amba_id = this->getStateIdAmba(state);
	const StatePC state_pc = this->packStatePc(state, pc);
	const TranslationBlockMetadata tb_meta = this->m_translation_block_metadata[state_pc];

	return (BasicBlockMetadata) {
		.symbolic_state_id = amba_id,
		.basic_block_vaddr = pc,
		.basic_block_generation = tb_meta.generation.val,
		.basic_block_elf_vaddr = tb_meta.elf_vaddr,
		.basic_block_content = tb_meta.content,
	};
}

}
