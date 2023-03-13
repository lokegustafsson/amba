#![allow(unsafe_code)]

pub mod graph;
pub mod lin_graph;
pub mod control_flow;

use control_flow::ControlFlowGraph;

#[no_mangle]
pub extern "C" fn rust_create_control_flow_graph() -> *mut ControlFlowGraph {
	// Allocate a ControlFlowGraph by leaking a Box
	let b = Box::new(ControlFlowGraph::new());
	Box::into_raw(b)
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_control_flow_graph(ptr: *mut ControlFlowGraph) {
	// Deallocate and run destructors by converting it into a Box and letting std handle it
	let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub extern "C" fn rust_main() -> std::ffi::c_int {
	println!("Hello world");
	let p = rust_create_control_flow_graph();
	0
}
