use autocxx::prelude::*;

autocxx::include_cpp! {
	#include "libs2ecore/include/s2e/CorePlugin.h"
	#include "libs2ecore/include/s2e/Plugin.h"
	#include "libs2ecore/include/s2e/S2EExecutionState.h"
	safety!(unsafe_ffi)
}
