autocxx::include_cpp! {
	#include "s2e/CorePlugin.h"
	#include "s2e/Plugin.h"
	#include "s2e/S2EExecutionState.h"
	#include "s2e/S2E.h"
	#include "s2e/ConfigFile.h"
	#include "s2e/Utils.h"
	#include "RustPlugin.h"

	safety!(unsafe_ffi)

	generate!("s2e::S2E")
	generate!("s2e::Plugin")
	generate!("s2e::CorePlugin")
	generate!("s2e::ConfigFile")
	generate!("s2e::ExecutionSignal")
	generate!("s2e::S2EExecutionState")
	generate!("TranslationBlock")
	generate!("s2e::plugins::RustPlugin")
}

pub mod rust_plugin;

pub mod types {
	pub mod s2e {
		pub use crate::ffi::s2e::*;
	}

	pub use crate::ffi::{s2e::plugins::RustPlugin as RawRustPlugin, *};

	pub use crate::rust_plugin::RustPlugin;
}

extern "C" {
	fn hello_cpp();
}

pub mod wrappers {
	pub fn hello_cpp() {
		unsafe { crate::hello_cpp() }
	}
}
