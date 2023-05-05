#include <thread>
#include <chrono>
#include <tuple>
#include <vector>
#include <unordered_set>

#include <s2e/S2E.h>
#include <klee/Searcher.h>

#include "StatePrioritisation.h"
#include "Amba.h"
#include "LibambaRs.h"
#include "s2e/S2EExecutionState.h"

namespace state_prioritisation {

// These pointers are not a race condition because the thread has to
// join before the AmbaPlugin fields can be destructed
void ipcReceiver(Ipc *ipc, bool *active, s2e::S2E *s2e) {
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

		const IdSet to_prioritise_ids = IdSet(receive_buffer.begin(), receive_buffer.end());

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
		auto old_searcher = executor.getSearcher();
		executor.setSearcher(new_searcher);
		if (old_searcher) {
			delete old_searcher;
		}
	}

	*amba::debug_stream() << "Exited ipc receiver thread\n";
}

}
