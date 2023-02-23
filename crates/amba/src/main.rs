use std::{pin::Pin, str::Utf8Error};

// use autocxx::prelude::*;
// use autocxx::subclass::*;
use cxx::let_cxx_string;
use s2e::{
	types::{
		s2e::{ExecutionSignal, S2EExecutionState},
		RawRustPlugin, RustPlugin, TranslationBlock,
	},
	wrappers::hello_cpp,
};

fn main() {
	hello_cpp();
	println!("Hello from Rust!");
}

unsafe trait ToRust {
	type Derefed;

	unsafe fn to_rust<'a>(self) -> Pin<&'a mut Self::Derefed>;
}

unsafe impl<T> ToRust for *mut T {
	type Derefed = T;

	unsafe fn to_rust<'a>(self) -> Pin<&'a mut T> {
		Pin::new_unchecked(&mut *self)
	}
}

#[no_mangle]
pub unsafe extern "C" fn initialise(this: *mut RawRustPlugin) {
	// Wrapped in an inner function because we need to use Result
	let f = || -> Result<(), Utf8Error> {
		let mut this = RustPlugin::from_raw_ptr(this);

		let_cxx_string!(
			key1 =
				this.as_super()
					.pin()
					.getConfigKey()
					.to_str()
					.map(str::to_owned)? + ".traceBlockTranslation"
		);
		let val = this
			.as_super()
			.pin()
			.s2e()
			.to_rust()
			.getConfig()
			.to_rust()
			.getBool(&key1, false, std::ptr::null_mut::<bool>()); // Default arguments from header
		this.pin().setTraceBlockTranslation(val);

		let_cxx_string!(
			key2 =
				this.as_super()
					.pin()
					.getConfigKey()
					.to_str()
					.map(str::to_owned)? + ".traceBlockExecution"
		);
		let val = this
			.as_super()
			.pin()
			.s2e()
			.to_rust()
			.getConfig()
			.to_rust()
			.getBool(&key2, false, std::ptr::null_mut::<bool>()); // Default arguments from header
		this.pin().setTraceBlockExecution(val);

		let _ = this
			.as_super()
			.pin()
			.s2e()
			.to_rust()
			.getCorePlugin()
			.to_rust()
			.onTranslateBlockStart
			.connect(_);

		// this->m_traceBlockTranslation = s2e()->getConfig()->getBool(getConfigKey() + ".traceBlockTranslation");
		// this->m_traceBlockExecution = s2e()->getConfig()->getBool(getConfigKey() + ".traceBlockExecution");
		// s2e()->getCorePlugin()->onTranslateBlockStart.connect(sigc::mem_fun(*this, &Example::slotTranslateBlockStart));
		todo!()
	};

	if let Err(e) = f() {
		eprintln!("Error during init: {e}\n({e:?})");
	}
}

#[no_mangle]
pub unsafe extern "C" fn slot_translate_block_start(
	_this: *mut RawRustPlugin,
	_e: *mut ExecutionSignal,
	_state: *mut S2EExecutionState,
	_tb: *mut TranslationBlock,
	_pc: u64,
) {
	// let this = Pin::new_unchecked(&mut *this);
	// if (*this).m_traceBlockTranslation {
	// getDebugStream(state) << "Translating block at " << hexval(pc) << "\n";
	// }

	// if (this.m_traceBlockExecution) {
	// this.signal.connect(sigc::mem_fun(*this, &RustPlugin::slotExecuteBlockStart));
	// }
	todo!()
}

#[no_mangle]
pub unsafe extern "C" fn slot_execute_block_start(
	_this: *mut RawRustPlugin,
	_state: *mut S2EExecutionState,
	_pc: u64,
) {
	// let this = Pin::new_unchecked(&mut *this);
	todo!()
	// getDebugStream(state) << "Executing block at " << hexval(pc) << "\n";
}
