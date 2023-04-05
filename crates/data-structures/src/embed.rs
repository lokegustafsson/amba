use fastrand::Rng;
use glam::DVec2;
use ipc::GraphIpc;

#[derive(Clone, Debug)]
pub struct Graph2D {
	pub nodes: Vec<Node2D>,
	pub edges: Vec<(usize, usize)>,
	pub min: DVec2,
	pub max: DVec2,
}
#[derive(Clone, Debug)]
pub struct Node2D {
	pub pos: DVec2,
}

impl Graph2D {
	pub fn empty() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
			min: DVec2::ZERO,
			max: DVec2::ZERO,
		}
	}

	pub fn embedding_of(graph: GraphIpc) -> Self {
		let mut nodes = vec![DVec2::ZERO; graph.metadata.len()];
		let mut node_push = vec![DVec2::ZERO; nodes.len()];
		let rng = Rng::with_seed(0);

		const NOISE: f64 = 0.3;
		const ATTRACTION: f64 = 0.2;
		const REPULSION: f64 = 1.0;
		const REPULSION_RADIUS: f64 = 0.02;
		const DOWNPUSH: f64 = 0.2;
		const ITERS: usize = 300;

		// NOTE: Insanely slow for now (100 * num_nodes^2 iterations)
		for temperature in (0..ITERS).map(|t| (t as f64 / ITERS as f64).powi(2)).rev() {
			node_push.fill_with(|| {
				DVec2 {
					x: rng.f64(),
					y: rng.f64() - DOWNPUSH,
				} * (NOISE * temperature)
			});
			for &(a, b) in &graph.edges {
				let push = ATTRACTION * (nodes[b] - nodes[a]);
				node_push[a] += push;
				node_push[b] -= push;
			}
			// TODO: Replace with quadtree approximation, a la Barnes-Hut
			for a in 0..nodes.len() {
				for b in 0..nodes.len() {
					let a_to_b = nodes[b] - nodes[a];
					let push = REPULSION * a_to_b / (REPULSION_RADIUS + a_to_b.length_squared());
					node_push[a] -= push;
					node_push[b] += push;
				}
			}
			for i in 1..nodes.len() {
				nodes[i] += node_push[i] - node_push[0];
				assert!(nodes[i].is_finite());
			}
		}
		Self {
			min: nodes
				.iter()
				.copied()
				.reduce(DVec2::min)
				.unwrap_or(DVec2::ZERO),
			max: nodes
				.iter()
				.copied()
				.reduce(DVec2::max)
				.unwrap_or(DVec2::ZERO),
			nodes: nodes.into_iter().map(|pos| Node2D { pos }).collect(),
			edges: graph.edges,
		}
	}
}
