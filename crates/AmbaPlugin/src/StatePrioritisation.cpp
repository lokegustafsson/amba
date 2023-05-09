#include <s2e/S2E.h>
#include <s2e/S2EExecutionState.h>
#include <klee/Searcher.h>

#include <thread>
#include <chrono>
#include <tuple>
#include <vector>
#include <unordered_set>
#include <atomic>

#include "StatePrioritisation.h"
#include "Amba.h"
#include "LibambaRs.h"

namespace state_prioritisation {

// These pointers are guaranteed to live long enough due to this
// thread being owned by the same object as owns this thread
void ipcReceiver(
	Ipc *ipc,
	std::atomic<bool> *active,
	s2e::S2E *s2e,
	std::mutex *dead_states_lock,
	std::unordered_set<i32> *dead_states,
	std::atomic<klee::Searcher *> *next_searcher
) {
	using IdSet = std::unordered_set<i32>;
	using StateSet = std::unordered_set<klee::ExecutionState *>;

	std::vector<i32> receive_buffer {};

	while (*active) {
		receive_buffer.clear();

		bool received = rust_ipc_receive_message(ipc, &receive_buffer);
		if (!received) {
			std::this_thread::sleep_for(std::chrono::milliseconds(200));
			continue;
		}

		const IdSet to_prioritise_ids = ([&]() {
			IdSet s {};
			dead_states_lock->lock();

			for (auto state : receive_buffer) {
				if (!dead_states->contains(state)) {
					s.insert(state);
				}
			}

			dead_states_lock->unlock();
			return s;
		})();

		auto &executor = *s2e->getExecutor();
		auto new_searcher = new klee::DFSSearcher();

		const StateSet &all_states = executor.getStates();
		const StateSet to_add = ([&]() {
			StateSet add {};

			for (const auto state : all_states) {
				const auto s2e_state = dynamic_cast<s2e::S2EExecutionState *>(state);
				const auto id = s2e_state->getGuid();

				if (to_prioritise_ids.contains(id)) {
					add.insert(state);
				}
			}

			return add;
		})();

		new_searcher->update(nullptr, to_add, {});

		klee::Searcher *expected;
		while (true) {
			expected = next_searcher->load();
			bool loaded = next_searcher->compare_exchange_weak(expected, new_searcher);
			if (loaded) {
				break;
			}
		}

		if (expected != nullptr) {
			delete expected;
		}
	}

	*amba::debug_stream() << "Exited ipc receiver thread\n";
}

}
