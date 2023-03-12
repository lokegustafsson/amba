use crate::graph::{Block, BlockId, Graph};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlFlowGraph {
	graph: Graph,
	compressed_graph: Graph,
	last: BlockId,
}

impl ControlFlowGraph {
	pub fn new() -> Self {
		Default::default()
	}
}
