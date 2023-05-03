#include <thread>
#include <chrono>
#include <vector>

#include "StatePrioritisation.h"
#include "Amba.h"
#include "LibambaRs.h"

namespace state_prioritisation {

// These pointers are not a race condition because the thread has to
// join before the AmbaPlugin fields can be destructed
void ipcReceiver(Ipc *ipc, bool *active, s2e::S2E *s2e) {
	std::vector<u32> receive_buffer {};

	while (*active) {
		receive_buffer.clear();
		bool received = rust_ipc_receive_message(ipc, &receive_buffer);

		std::this_thread::sleep_for(std::chrono::milliseconds(200));

		if (!received) {
			continue;
		}

		auto printer = amba::debug_stream();
		*printer << "Prio states: ";
		for (auto& state : receive_buffer) {
			*printer << state << ", ";
		}
		*printer << '\n';
		continue;

		s2e->getExecutor()->suspendState(
			nullptr
		);
	}

	*amba::debug_stream() << "Exited ipc receiver thread\n";
}

}
