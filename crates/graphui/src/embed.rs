use data_structures::{GraphIpc, NodeMetadata};
use fastrand::Rng;
use glam::DVec2;

#[derive(Clone, Debug)]
pub struct Graph2D {
	pub node_metadata: Vec<NodeMetadata>,
	pub node_positions: Vec<DVec2>,
	pub edges: Vec<(usize, usize)>,
	pub min: DVec2,
	pub max: DVec2,
	pub gui_zoom: f32,
	pub gui_pos: emath::Vec2,
}

#[derive(Clone, Copy)]
pub struct EmbeddingParameters {
	pub noise: f64,
	pub attraction: f64,
	pub repulsion: f64,
	pub gravity: f64,
}
impl EmbeddingParameters {
	pub const MAX_ATTRACTION: f64 = 0.2;
	pub const MAX_GRAVITY: f64 = 2.0;
	pub const MAX_NOISE: f64 = 20.0;
	pub const MAX_REPULSION: f64 = 2.0;
}
impl Default for EmbeddingParameters {
	fn default() -> Self {
		Self {
			noise: 0.0,
			attraction: 0.1,
			repulsion: 1.0,
			gravity: 0.5,
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
			gui_zoom: 1.0,
			gui_pos: emath::Vec2::ZERO,
		}
	}

	pub fn new(graph: GraphIpc, params: EmbeddingParameters) -> Self {
		if graph.metadata.is_empty() {
			return Self::empty();
		}

		let rng = &Rng::with_seed(0);
		let mut ret = Self {
			node_positions: {
				let mut adjacency_list = vec![Vec::new(); graph.metadata.len()];
				for &(a, b) in &graph.edges {
					adjacency_list[a].push(b);
				}

				let mut node_depth = vec![usize::MAX; graph.metadata.len()];
				node_depth[0] = 0;
				let mut stack = vec![0];
				while let Some(i) = stack.pop() {
					for &e in &adjacency_list[i] {
						if node_depth[e] == usize::MAX {
							node_depth[e] = node_depth[i] + 1;
							stack.push(e);
						}
					}
				}

				(0..graph.metadata.len())
					.map(|i| (random_dvec2(rng) + DVec2::Y) * node_depth[i] as f64)
					.collect()
			},
			node_metadata: graph.metadata,
			edges: graph.edges,
			min: DVec2::ZERO,
			max: DVec2::ZERO,
			gui_zoom: 1.0,
			gui_pos: emath::Vec2::ZERO,
		};
		ret.run_layout_iterations(100, params);
		ret
	}

	pub fn run_layout_iterations(&mut self, iterations: usize, params: EmbeddingParameters) {
		if self.node_positions.is_empty() {
			return;
		}
		let mut node_velocity = vec![DVec2::ZERO; self.node_positions.len()];
		let mut node_accel = vec![DVec2::ZERO; self.node_positions.len()];
		let rng = &Rng::new();

		// NOTE: Insanely slow for now (`iterations` * `self.node_positions.len()`^2)
		for temperature in (0..iterations)
			.map(|t| (t as f64 / iterations as f64).powi(2))
			.rev()
		{
			node_accel.fill(DVec2::ZERO);
			// Edges attract with `F \propto D^1.2`
			for &(a, b) in &self.edges {
				const EDGE_ATTRACT_EXPONENT: f64 = 0.2;
				let delta = self.node_positions[b] - self.node_positions[a];
				let scale = delta.length().powf(EDGE_ATTRACT_EXPONENT);
				let push = params.attraction * delta * scale;
				node_accel[a] += push;
				node_accel[b] -= push;
			}
			// TODO: Replace with quadtree approximation, a la Barnes-Hut
			// Nodes repell with `F \propto D^-2`
			for a in 0..self.node_positions.len() {
				for b in 0..self.node_positions.len() {
					let a_to_b = self.node_positions[b] - self.node_positions[a];
					let push = params.repulsion * a_to_b / (1.0 + a_to_b.length().powi(3));
					node_accel[a] -= push;
					node_accel[b] += push;
				}
			}
			let a0 = node_accel[0];
			for i in 1..self.node_positions.len() {
				// Gravity
				node_accel[i] += DVec2::Y * params.gravity;
				node_accel[i] -= a0;
				// Opposite accel and velocity => exponentially reduce velocity
				if node_accel[i].dot(node_velocity[i]) > 0.0 {
					const VELOCITY_SPEEDUP: f64 = 1.10;
					node_velocity[i] *= VELOCITY_SPEEDUP;
				} else {
					const VELOCITY_SLOWDOWN: f64 = 0.9;
					node_velocity[i] *= VELOCITY_SLOWDOWN;
				}
				node_velocity[i] += node_accel[i];
				self.node_positions[i] +=
					node_velocity[i] + random_dvec2(rng) * (params.noise * temperature);

				assert!(node_accel[i].is_finite());
				assert!(node_velocity[i].is_finite());
				assert!(self.node_positions[i].is_finite());
			}
			let rotate_down_center_of_mass = {
				let center_of_mass = self.node_positions.iter().sum::<DVec2>();
				(2.0 * center_of_mass.project_onto(DVec2::ONE) - center_of_mass).normalize()
			};
			for i in 1..self.node_positions.len() {
				self.node_positions[i] = rotate_down_center_of_mass.rotate(self.node_positions[i]);
				node_velocity[i] = rotate_down_center_of_mass.rotate(node_velocity[i]);
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

fn random_dvec2(rng: &Rng) -> DVec2 {
	DVec2 {
		x: rng.f64(),
		y: rng.f64(),
	}
}
