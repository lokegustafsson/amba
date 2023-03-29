use crate::Graph;

pub struct Graph2D {
	nodes: Vec<Node2D>,
	edges: Vec<(usize, usize)>,
}
struct Node2D {
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

	pub fn embedding_of(graph: &Graph) -> Self {
		todo!()
	}
}
