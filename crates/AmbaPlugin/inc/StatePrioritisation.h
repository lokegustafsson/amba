#pragma once

#include <s2e/S2E.h>

#include "LibambaRs.h"

namespace state_prioritisation {

void ipcReceiver(Ipc *ipc, bool *active, s2e::S2E *s2e);

}
