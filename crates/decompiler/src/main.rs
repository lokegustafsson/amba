use s2e::{
	types::{
		s2e::{ExecutionSignal, Plugin, S2EExecutionState, S2E},
		TranslationBlock,
	},
	wrappers::hello_cpp,
};

fn main() {
	hello_cpp();
	println!("Hello from Rust!");
}

#[no_mangle]
pub unsafe extern "C" fn initialise() {
	todo!()
	// m_traceBlockTranslation = s2e()->getConfig()->getBool(getConfigKey() + ".traceBlockTranslation");
	// m_traceBlockExecution = s2e()->getConfig()->getBool(getConfigKey() + ".traceBlockExecution");
	// s2e()->getCorePlugin()->onTranslateBlockStart.connect(sigc::mem_fun(*this, &RustPlugin::slotTranslateBlockStart));
}

#[no_mangle]
pub unsafe extern "C" fn slot_translate_block_start(
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
pub unsafe extern "C" fn slot_execute_block_start(state: *mut S2EExecutionState, pc: u64) {
	todo!()
	// getDebugStream(state) << "Executing block at " << hexval(pc) << "\n";
}
