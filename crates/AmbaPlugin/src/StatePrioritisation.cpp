#include "StatePrioritisation.h"

namespace state_prioritisation {

void ipcReceiver(IpcRx *ipc, std::shared_ptr<bool> active, s2e::S2E *s2e) {
	while (*active) {
	}
}

}
