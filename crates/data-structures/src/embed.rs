use crate::GraphIpc;

pub struct Graph2D {
	nodes: Vec<Node2D>,
	edges: Vec<(usize, usize)>,
}
pub struct Node2D {
	x: f64,
	y: f64,
}

impl Graph2D {
	pub fn empty() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
		}
	}

	pub fn embedding_of(graph: GraphIpc) -> Self {
		let nodes = todo!();
		Self {
			nodes,
			edges: graph.edges,
		}
	}
}
