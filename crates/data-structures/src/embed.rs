use std::borrow::Cow;

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

	pub fn new(graph: Cow<'_, GraphIpc>) -> Self {
		let (node_metadata, edges) = match graph {
			Cow::Borrowed(graph) => (graph.metadata.clone(), graph.edges.clone()),
			Cow::Owned(graph) => (graph.metadata, graph.edges),
		};
		let mut ret = Self {
			node_positions: vec![DVec2::ZERO; node_metadata.len()],
			node_metadata,
			edges,
			min: DVec2::ZERO,
			max: DVec2::ZERO,
		};
		ret.run_layout_iterations(100);
		ret
	}

	pub fn run_layout_iterations(&mut self, iterations: usize) {
		let mut node_push = vec![DVec2::ZERO; self.node_positions.len()];
		let rng = Rng::with_seed(0);

		const NOISE: f64 = 0.3;
		const ATTRACTION: f64 = 0.2;
		const REPULSION: f64 = 1.0;
		const REPULSION_RADIUS: f64 = 0.02;
		const DOWNPUSH: f64 = 0.2;

		// NOTE: Insanely slow for now (`iterations` * `self.node_positions.len()`^2)
		for temperature in (0..iterations)
			.map(|t| (t as f64 / iterations as f64).powi(2))
			.rev()
		{
			node_push.fill_with(|| {
				DVec2 {
					x: rng.f64(),
					y: rng.f64() - DOWNPUSH,
				} * (NOISE * temperature)
			});
			for &(a, b) in &self.edges {
				let push = ATTRACTION * (self.node_positions[b] - self.node_positions[a]);
				node_push[a] += push;
				node_push[b] -= push;
			}
			// TODO: Replace with quadtree approximation, a la Barnes-Hut
			for a in 0..self.node_positions.len() {
				for b in 0..self.node_positions.len() {
					let a_to_b = self.node_positions[b] - self.node_positions[a];
					let push = REPULSION * a_to_b / (REPULSION_RADIUS + a_to_b.length_squared());
					node_push[a] -= push;
					node_push[b] += push;
				}
			}
			for i in 1..self.node_positions.len() {
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
