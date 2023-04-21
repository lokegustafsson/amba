#pragma once

#include <s2e/S2E.h>

#include <memory>
#include <thread>

#include "LibambaRs.h"

namespace state_prioritisation {

void ipcReceiver(IpcRx *ipc, std::shared_ptr<bool> active, s2e::S2E *s2e);

}
