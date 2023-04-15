#include "ControlFlow.h"
#include "AmbaException.h"

namespace control_flow {

void updateControlFlowGraph(ControlFlowGraph *cfg, Packed from, Packed to) {
	rust_update_control_flow_graph(
		cfg,
		from.val,
		to.val
	);
}

ControlFlow::ControlFlow(std::string name)
	: m_name(name)
	, m_cfg(rust_new_control_flow_graph())
{}

ControlFlow::~ControlFlow() {
	rust_print_graph_size(this->m_name.c_str(), this->m_cfg);
	rust_free_control_flow_graph(this->m_cfg);
}

u64 ControlFlow::getBlockId(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const i32 state = s2e_state->getID();
	const u64 gen = this->m_uuids[state];
	const u64 vaddr = pc;

	const u64 packed
		= (0x0000'FFFF'FFFF'FFFF & vaddr)
		| (0x000F'0000'0000'0000 & ((u64) gen.val << 48))
		| (0xFFF0'0000'0000'0000 & ((u64) state.val << 52));

	const u64 vaddr_ =  packed & 0x0000'FFFF'FFFF'FFFF;
	const u64 gen_   = (packed & 0x000F'0000'0000'0000) >> 48;
	const u64 state_ = (packed & 0xFFF0'0000'0000'0000) >> 52;

	AMBA_ASSERT(vaddr == vaddr_);
	AMBA_ASSERT(gen.val == gen_);
	AMBA_ASSERT((u64) state.val == state_);

	return packed;
}

void ControlFlow::translateBlockStart(
	s2e::ExecutionSignal *signal,
	s2e::S2EExecutionState *state,
	TranslationBlock *tb,
	u64 pc
) {
	const auto key = this->getBlockId(state, pc);
	++this->m_generations[key].val;
}

void ControlFlow::onBlockStart(
	s2e::S2EExecutionState *state,
	u64 pc
) {
	auto curr = this->getBlockId(state, pc);
	// Will insert 0 if value doesn't yet exist
	auto &last = this->m_last[(u64) state->getID()];
	rust_update_control_flow_graph(
		this->m_cfg,
		last,
		curr
	);
	last = curr;
}

void ControlFlow::onStateFork(
	s2e::S2EExecutionState *old_state,
	const std::vector<s2e::S2EExecutionState *> &new_states,
	const std::vector<klee::ref<klee::Expr>> &conditions
) {
	// The symbolic control flow graph ids are 64 bit values where the
	// lower 32 bits are the uuid of the state and the upper 32 bits
	// are the generation of reuse of that uuid.

	const i32 old_id = old_state->getID();
	const u64 from = this->m_uuids[old_id] << 32 | (u64) old_id;
	const u64 last_raw = this->m_last[from];

	for (auto &new_state : new_states) {
		const i32 new_id = new_state->getID();

		AMBA_ASSERT(new_id != old_id);

		const u64 to = (++this->m_uuids[new_id]) << 32 | (u64) new_id;
		this->m_last[to] = last_raw;

		rust_update_control_flow_graph(
			this->m_cfg,
			from,
			to
		);
	}
}

void ControlFlow::onStateMerge(
	s2e::S2EExecutionState *destination_state,
	s2e::S2EExecutionState *source_state
) {
	const i32 dest_id = destination_state->getID();
	const i32 src_id = source_state->getID();

	const u64 from_left = this->m_uuids[dest_id] << 32 | (u64) dest_id;
	const u64 from_right = this->m_uuids[src_id] << 32 | (u64) src_id;

	const u64 to = (++this->m_uuids[dest_id]) << 32 | (u64) dest_id;

	rust_update_control_flow_graph(
		this->m_cfg,
		from_left,
		to
	);
	rust_update_control_flow_graph(
		this->m_cfg,
		from_right,
		to
	);
}

const char *ControlFlow::getName() const {
	return this->m_name.c_str();
}

ControlFlowGraph *ControlFlow::cfg() {
	return this->m_cfg;
}

} // namespace control_flow
