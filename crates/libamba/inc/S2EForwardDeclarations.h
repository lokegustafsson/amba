#pragma once

#include "Numbers.h"

namespace sigc {
	template <typename RET, typename... PARAM_TYPES> class signal;
}

namespace s2e {
	class S2EExecutionState;
	typedef sigc::signal<void, S2EExecutionState *, u64> ExecutionSignal;
}
struct TranslationBlock;
struct CPUX86State;
