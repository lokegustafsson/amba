use fastrand::Rng;
use glam::DVec2;

use crate::{GraphIpc, NodeMetadata};

#[derive(Clone, Debug)]
pub struct Graph2D {
	pub node_metadata: Vec<NodeMetadata>,
	pub node_positions: Vec<DVec2>,
	pub edges: Vec<(usize, usize)>,
	pub min: DVec2,
	pub max: DVec2,
}

#[derive(Clone, Copy)]
pub struct EmbeddingParameters {
	pub noise: f64,
	pub attraction: f64,
	pub repulsion: f64,
	pub gravity: f64,
}
impl Default for EmbeddingParameters {
	fn default() -> Self {
		Self {
			noise: 10.0,
			attraction: 0.1,
			repulsion: 1.0,
			gravity: 0.0,
		}
	}
}

impl Graph2D {
	pub fn empty() -> Self {
		Self {
			node_metadata: Vec::new(),
			node_positions: Vec::new(),
			edges: Vec::new(),
			min: DVec2::ZERO,
			max: DVec2::ZERO,
		}
	}

	pub fn new(graph: GraphIpc, params: EmbeddingParameters) -> Self {
		let mut ret = Self {
			node_positions: vec![DVec2::ZERO; graph.metadata.len()],
			node_metadata: graph.metadata,
			edges: graph.edges,
			min: DVec2::ZERO,
			max: DVec2::ZERO,
		};
		ret.run_layout_iterations(100, params);
		ret
	}

	pub fn run_layout_iterations(&mut self, iterations: usize, params: EmbeddingParameters) {
		let mut node_push = vec![DVec2::ZERO; self.node_positions.len()];
		let rng = Rng::new();

		// NOTE: Insanely slow for now (`iterations` * `self.node_positions.len()`^2)
		for temperature in (0..iterations)
			.map(|t| (t as f64 / iterations as f64).powi(2))
			.rev()
		{
			node_push.fill_with(|| {
				DVec2 {
					x: rng.f64(),
					y: rng.f64(),
				} * (params.noise * temperature)
			});
			for &(a, b) in &self.edges {
				let push = params.attraction * (self.node_positions[b] - self.node_positions[a]);
				node_push[a] += push;
				node_push[b] -= push;
			}
			// TODO: Replace with quadtree approximation, a la Barnes-Hut
			for a in 0..self.node_positions.len() {
				for b in 0..self.node_positions.len() {
					let a_to_b = self.node_positions[b] - self.node_positions[a];
					let push = params.repulsion * a_to_b / (1.0 + a_to_b.length_squared());
					node_push[a] -= push;
					node_push[b] += push;
				}
			}
			for i in 1..self.node_positions.len() {
				node_push[i] +=
					DVec2::Y * (params.gravity * self.node_positions[i].y.max(1.0).log2());
				self.node_positions[i] += node_push[i] - node_push[0];
				assert!(self.node_positions[i].is_finite());
			}
		}
		self.min = self
			.node_positions
			.iter()
			.copied()
			.reduce(DVec2::min)
			.unwrap_or(DVec2::ZERO);
		self.max = self
			.node_positions
			.iter()
			.copied()
			.reduce(DVec2::max)
			.unwrap_or(DVec2::ZERO);
	}
}
