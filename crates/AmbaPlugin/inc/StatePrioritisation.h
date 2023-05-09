#pragma once

#include <s2e/S2E.h>

#include <atomic>

#include "LibambaRs.h"

namespace state_prioritisation {

void ipcReceiver(
	Ipc *ipc,
	std::atomic<bool> *active,
	s2e::S2E *s2e,
	std::mutex *dead_states_lock,
	std::unordered_set<i32> *dead_states,
	std::atomic<klee::Searcher *> *next_searcher
);

}
