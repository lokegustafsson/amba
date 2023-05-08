#pragma once

#include <s2e/S2E.h>

#include "LibambaRs.h"

namespace state_prioritisation {

void ipcReceiver(
	Ipc *ipc,
	bool *active,
	s2e::S2E *s2e,
	std::mutex *dead_states_lock,
	std::unordered_set<i32> *dead_states
);

}
