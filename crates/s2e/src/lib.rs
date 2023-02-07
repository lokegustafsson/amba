use autocxx::prelude::*;

autocxx::include_cpp! {
	#include "s2e/CorePlugin.h"
	#include "s2e/Plugin.h"
	#include "s2e/S2EExecutionState.h"
	safety!(unsafe_ffi)
}
