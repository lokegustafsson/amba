#include <vector>

#include "ControlFlow.h"
#include "AmbaException.h"

namespace control_flow {

NodeMetadataFFI StateMetadata::into_ffi() const {
	return (NodeMetadataFFI) {
		.metadata_type = 0,
		.symbolic_state_id = (u32) this->symbolic_state_id.val,
		.basic_block_vaddr = 0,
		.basic_block_generation = 0,
		.basic_block_elf_vaddr = 0,
		.basic_block_content = std::make_unique<std::vector<u8>>(),
	};
}

NodeMetadataFFI BasicBlockMetadata::into_ffi() const {
	return (NodeMetadataFFI) {
		.metadata_type = 1,
		.symbolic_state_id = (u32) this->symbolic_state_id.val,
		.basic_block_vaddr = this->basic_block_vaddr,
		.basic_block_generation = this->basic_block_generation,
		.basic_block_elf_vaddr = this->basic_block_elf_vaddr,
		.basic_block_content = std::make_unique<std::vector<u8>>(this->basic_block_content),
	};
}

StateIdS2E getStateIdS2E(s2e::S2EExecutionState *state) {
	return StateIdS2E(state->getGuid());
}

ControlFlow::ControlFlow(std::string name)
	: m_name(name)
{}

const char *ControlFlow::getName() const {
	return this->m_name.c_str();
}

u64 ControlFlow::states() const {
	return this->state_count;
}

std::vector<NodeMetadataFFIPair> &ControlFlow::edges() {
	return this->m_new_edges;
}

StateIdAmba ControlFlow::getStateIdAmba(StateIdS2E id) {
	auto& amba_id = this->m_states[id];
	if (amba_id == 0) {
		this->state_count++;
		amba_id.val = this->state_count;
	}
	return amba_id;
}

void ControlFlow::incrementStateIdAmba(StateIdS2E id) {
	this->state_count++;
	auto& amba_id = this->m_states[id];
	amba_id.val = this->state_count;
}

} // namespace control_flow
