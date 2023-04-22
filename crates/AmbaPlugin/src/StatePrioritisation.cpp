#include <vector>

#include "StatePrioritisation.h"
#include "LibambaRs.h"

namespace state_prioritisation {

// These pointers are not a race condition because the thread has to
// join before the AmbaPlugin fields can be destructed
void ipcReceiver(IpcRx *ipc, bool *active, s2e::S2E *s2e) {
	std::vector<u32> receive_buffer {};
	while (*active) {
		receive_buffer.clear();
		bool received = rust_ipc_receive_message(ipc, &receive_buffer);
		if (!received) {
			continue;
		}

		s2e->getExecutor()->suspendState(
			nullptr
		);
	}
}

}
