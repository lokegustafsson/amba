#![allow(unused_variables, dead_code, unused_imports)]

use autocxx::{prelude::*, subclass::prelude::*};

autocxx::include_cpp! {
	#include "s2e/CorePlugin.h"
	#include "s2e/Plugin.h"
	#include "s2e/S2EExecutionState.h"
	safety!(unsafe_ffi)

	generate!("s2e::Plugin")
	generate!("s2e::S2E")
	generate!("s2e::ExecutionSignal")
	generate!("TranslationBlock")
	generate!("s2e::S2EExecutionState")
}

use crate::ffi::{
	s2e::{ExecutionSignal, Plugin, S2EExecutionState, S2E},
	TranslationBlock,
};

#[no_mangle]
pub unsafe extern fn initialise() {
	todo!()
	// m_traceBlockTranslation = s2e()->getConfig()->getBool(getConfigKey() + ".traceBlockTranslation");
	// m_traceBlockExecution = s2e()->getConfig()->getBool(getConfigKey() + ".traceBlockExecution");
	// s2e()->getCorePlugin()->onTranslateBlockStart.connect(sigc::mem_fun(*this, &RustPlugin::slotTranslateBlockStart));
}

#[no_mangle]
pub unsafe extern fn slot_translate_block_start(
	e: *mut ExecutionSignal,
	state: *mut S2EExecutionState,
	tb: *mut TranslationBlock,
	pc: u64,
) {
	todo!()
	// if (m_traceBlockTranslation) {
	// getDebugStream(state) << "Translating block at " << hexval(pc) << "\n";
	// }

	// if (m_traceBlockExecution) {
	// signal->connect(sigc::mem_fun(*this, &RustPlugin::slotExecuteBlockStart));
	// }
}

#[no_mangle]
pub unsafe extern fn slot_execute_block_start(state: *mut S2EExecutionState, pc: u64) {
	todo!()
	// getDebugStream(state) << "Executing block at " << hexval(pc) << "\n";
}
