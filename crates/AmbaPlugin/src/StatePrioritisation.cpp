#include <vector>

#include "StatePrioritisation.h"
#include "LibambaRs.h"

namespace state_prioritisation {

void ipcReceiver(IpcRx *ipc, std::shared_ptr<bool> active, s2e::S2E *s2e) {
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
