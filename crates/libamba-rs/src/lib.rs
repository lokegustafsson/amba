use std::{rc::Rc, default::Default};

type Set<T> = std::collections::HashSet<T>;
type Map<K, V> = std::collections::HashMap<K, V>;
type BlockId = u64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlFlowGraph {
	graph: Map<u64, Block>,
	// merged_nodes: Map<u64, Rc<Set<Block>>>,
	last: BlockId
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Block {
	id: BlockId,
	from: Set<BlockId>,
	to: Set<BlockId>,
}

#[no_mangle]
pub extern "C" fn rust_create_control_flow_grapgh() -> *mut ControlFlowGraph {
	// Allocate a ControlFlowGraph by leaking a Box
	let b = Box::new(ControlFlowGraph::default());
	Box::into_raw(b) as _
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_control_flow_grapgh(ptr: *mut ControlFlowGraph) {
	// Deallocate and run destructors by converting it into a Box and letting std handle it
	let _ = Box::from_raw(ptr);
}


#[no_mangle]
pub extern "C" fn four() -> i32 {
	4
}
