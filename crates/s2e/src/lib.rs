use autocxx::{prelude::*, subclass::prelude::*};

autocxx::include_cpp! {
	#include "s2e/CorePlugin.h"
	#include "s2e/Plugin.h"
	#include "s2e/S2EExecutionState.h"
	safety!(unsafe_ffi)

	generate!("s2e::Plugin")
	generate!("s2e::S2E")
	generate!("s2e::ExecutionSignal")
	generate!("s2e::S2EExecutionState")
	generate!("TranslationBlock")
}

use crate::ffi::{
	s2e::{ExecutionSignal, Plugin, S2EExecutionState, S2E},
	TranslationBlock,
};

pub mod types {
	pub mod s2e {
		pub use crate::ffi::s2e::*;
	}

	pub use crate::ffi::*;
}

extern "C" {
	fn hello_cpp();
}

pub mod wrappers {
	pub fn hello_cpp() {
		unsafe { crate::hello_cpp() }
	}
}
