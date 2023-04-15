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

StatePC ControlFlow::toAlias(UidS2E uid, u64 pc) {
	return this->m_uuids[uid].val << 32 | (u64) uid.val;
}

Packed ControlFlow::getBlockId(
	s2e::S2EExecutionState *s2e_state,
	u64 pc
) {
	const UidS2E state = UidS2E(s2e_state->getID());
	const Generation gen = this->m_generations[state];
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

	return Packed(packed);
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
	const Packed curr = this->getBlockId(state, pc);
	// Will insert 0 if value doesn't yet exist
	auto &last = this->m_last[curr];
	updateControlFlowGraph(
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
	const UidS2E old_id = UidS2E(old_state->getID());

	const Packed from = this->getBlockId(old_state, 0);
	const u64 last_raw = this->m_last[from.val];

	for (auto &new_state : new_states) {
		const UidS2E new_id = UidS2E(new_state->getID());

		if (new_id == old_id) {
			++this->m_uuids[new_id].val;
		}

		const Packed to = this->getBlockId(new_state, 0);
		this->m_last[to.val] = last_raw;

		updateControlFlowGraph(
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
	const UidS2E dest_id = UidS2E(destination_state->getID());

	const Packed from_left = this->getBlockId(destination_state, 0);
	const Packed from_right = this->getBlockId(source_state, 0);

	++this->m_uuids[dest_id].val;
	const Packed to = this->getBlockId(destination_state, 0);

	updateControlFlowGraph(
		this->m_cfg,
		from_left,
		to
	);
	updateControlFlowGraph(
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
