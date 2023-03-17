pub mod control_flow;
mod disjoint_sets;
pub mod graph;

#[allow(unsafe_code, clippy::missing_safety_doc)]
mod ffi {
	use crate::control_flow::ControlFlowGraph;

	/// Create a newly allocated `ControlFlowGraph` and return an
	/// owning raw pointer. This pointer may only be freed with
	/// the `rust_free_control_flow_graph` function.
	#[no_mangle]
	pub extern "C" fn rust_new_control_flow_graph() -> *mut ControlFlowGraph {
		Box::into_raw(Box::new(ControlFlowGraph::new()))
	}

	/// Free a `ControlFlowGraph` allocated by
	/// `rust_new_control_flow_graph`. After this fuction has been
	/// called the pointer may not be used again.
	#[no_mangle]
	pub unsafe extern "C" fn rust_free_control_flow_graph(ptr: *mut ControlFlowGraph) {
		let _ = Box::from_raw(ptr);
	}

	/// Wrapper around `ControlFlowGraph::update`. May only be
	/// called with a pointer allocated by
	/// `rust_new_control_flow_graph`. Returns true if the graph
	/// has changed.
	#[no_mangle]
	pub unsafe extern "C" fn rust_update_control_flow_graph(
		ptr: *mut ControlFlowGraph,
		from: u64,
		to: u64,
	) -> bool {
		false
	}

	#[no_mangle]
	pub unsafe extern "C" fn rust_print_graph_size(ptr: *mut ControlFlowGraph) {}

	#[no_mangle]
	pub extern "C" fn rust_main() -> std::ffi::c_int {
		println!("Hello world");
		0
	}
}
