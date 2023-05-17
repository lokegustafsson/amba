#include <vector>

#include "ControlFlow.h"
#include "AmbaException.h"
#include "LibambaRs.h"

namespace control_flow {

NodeMetadataFFI StateMetadata::into_ffi() const {
	return (NodeMetadataFFI) {
		.metadata_type = 0,
		.amba_state_id = (u32) this->amba_state_id.val,
		.s2e_state_id = (i32) this->s2e_state_id.val,
		.basic_block_vaddr = 0,
		.basic_block_generation = 0,
		.basic_block_elf_vaddr = 0,
		.basic_block_content = std::make_unique<std::vector<u8>>(),
		.state_concrete_inputs = control_flow::concreteInputsIntoFFI(this->concrete_inputs),
	};
}

NodeMetadataFFI BasicBlockMetadata::into_ffi() const {
	return (NodeMetadataFFI) {
		.metadata_type = 1,
		.amba_state_id = (u32) this->symbolic_state_id.val,
		.s2e_state_id = 0,
		.basic_block_vaddr = this->basic_block_vaddr,
		.basic_block_generation = this->basic_block_generation,
		.basic_block_elf_vaddr = this->basic_block_elf_vaddr,
		.basic_block_content = std::make_unique<std::vector<u8>>(this->basic_block_content),
		.state_concrete_inputs = {
			.names = std::make_unique<std::vector<std::string>>(),
			.byte_counts = std::make_unique<std::vector<i32>>(),
			.bytes = std::make_unique<std::vector<u8>>(),
		},
	};
}

StateIdS2E getStateIdS2E(s2e::S2EExecutionState *state) {
	return StateIdS2E(state->getGuid());
}

ConcreteInputsFFI concreteInputsIntoFFI(ConcreteInputs inputs) {
	std::vector<std::string> names;
	std::vector<i32> byte_counts;
	std::vector<u8> bytes;
	for (auto input : inputs) {
		std::string name = input.first;
		auto this_bytes = input.second;
		i32 byte_count = (i32) bytes.size();

		names.push_back(name);
		byte_counts.push_back(byte_count);
		byte_counts.push_back(byte_count);

		for (auto byte : this_bytes) {
			bytes.push_back(byte);
		}
	}
	return (ConcreteInputsFFI) {
		.names = std::make_unique<std::vector<std::string>>(names),
		.byte_counts = std::make_unique<std::vector<i32>>(byte_counts),
		.bytes = std::make_unique<std::vector<u8>>(bytes),
	};
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
