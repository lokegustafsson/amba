#pragma once

#include "s2e/CorePlugin.h"
#include "s2e/Plugin.h"
#include "s2e/S2EExecutionState.h"

extern "C" {

void initialise();
void slot_translate_block_start(
	s2e::ExecutionSignal*,
	s2e::S2EExecutionState*,
	TranslationBlock*,
	uint64_t
);
void slot_execute_block_start(s2e::S2EExecutionState *, uint64_t);

}
