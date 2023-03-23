pub mod control_flow;

#[allow(unsafe_code, clippy::missing_safety_doc)]
mod ffi {
	use std::{os::unix::net::UnixStream, sync::Mutex};

	use crate::control_flow::ControlFlowGraph;

	type Ipc = ipc::Ipc<&'static std::os::unix::net::UnixStream>;

	/// Create a newly allocated `ControlFlowGraph` and return an
	/// owning raw pointer. This pointer may only be freed with
	/// the `rust_free_control_flow_graph` function.
	#[no_mangle]
	pub extern "C" fn rust_new_control_flow_graph() -> *mut Mutex<ControlFlowGraph> {
		Box::into_raw(Box::new(Mutex::new(ControlFlowGraph::new())))
	}

	/// Free a `ControlFlowGraph` allocated by
	/// `rust_new_control_flow_graph`. After this fuction has been
	/// called the pointer may not be used again.
	#[no_mangle]
	pub unsafe extern "C" fn rust_free_control_flow_graph(ptr: *mut Mutex<ControlFlowGraph>) {
		let _ = Box::from_raw(ptr);
	}

	/// Wrapper around `ControlFlowGraph::update`. May only be
	/// called with a pointer allocated by
	/// `rust_new_control_flow_graph`. Returns true if the graph
	/// has changed.
	#[no_mangle]
	pub unsafe extern "C" fn rust_update_control_flow_graph(
		ptr: *mut Mutex<ControlFlowGraph>,
		from: u64,
		to: u64,
	) -> bool {
		let mutex = &*ptr;
		let mut cfg = mutex.lock().unwrap();
		cfg.update(from, to)
	}

	#[no_mangle]
	pub unsafe extern "C" fn rust_print_graph_size(ptr: *mut Mutex<ControlFlowGraph>) {
		let mutex = &*ptr;
		let cfg = mutex.lock().unwrap();
		println!("{cfg}");
	}

	/// Initialize `Ipc` and return an owning raw pointer. If initialization
	/// fails, a null pointer is returned.
	#[no_mangle]
	pub extern "C" fn rust_ipc_new() -> *mut Ipc {
		match UnixStream::connect("amba-ipc.socket") {
			Ok(stream) => {
				let stream = Box::leak(Box::new(stream));
				Box::leak(Box::new(Ipc::new(&*stream)))
			}
			Err(err) => {
				println!("libamba failed to connect to IPC socket: {err:?}");
				std::ptr::null_mut()
			}
		}
	}

	#[no_mangle]
	pub extern "C" fn rust_ipc_send_graph(ipc: &mut Ipc, graph: &mut ControlFlowGraph) {
		ipc.blocking_send(&ipc::IpcMessage::Ping)
			.unwrap_or_else(|err| println!("libamba ipc error: {err:?}"));
	}

	#[no_mangle]
	pub extern "C" fn rust_main() -> std::ffi::c_int {
		println!("Hello world");
		0
	}
}
