use s2e::{
	types::{
		s2e::{ExecutionSignal, S2EExecutionState},
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
	_e: *mut ExecutionSignal,
	_state: *mut S2EExecutionState,
	_tb: *mut TranslationBlock,
	_pc: u64,
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
pub unsafe extern "C" fn slot_execute_block_start(_state: *mut S2EExecutionState, _pc: u64) {
	todo!()
	// getDebugStream(state) << "Executing block at " << hexval(pc) << "\n";
}
