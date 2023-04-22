#pragma once

#include <s2e/S2E.h>

#include <memory>
#include <thread>

#include "LibambaRs.h"

namespace state_prioritisation {

void ipcReceiver(IpcRx *ipc, bool *active, s2e::S2E *s2e);

}
