pub mod control_flow;
pub mod disjoint_sets;
pub mod graph;
pub mod small_set;

#[allow(unsafe_code, clippy::missing_safety_doc)]
mod ffi {
	use std::sync::Mutex;

	use crate::control_flow::ControlFlowGraph;

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

	#[no_mangle]
	pub extern "C" fn rust_main() -> std::ffi::c_int {
		println!("Hello world");
		0
	}
}
