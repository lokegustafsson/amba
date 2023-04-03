use std::mem;

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
	pub repulsion_approximation: f64,
	pub statistic_updates_per_second: f64,
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
			repulsion_approximation: 0.2,
			statistic_updates_per_second: 1.0,
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

		let mut ret = Self {
			node_positions: Self::initial_node_positions(&graph.metadata, &graph.edges),
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

	fn initial_node_positions(
		metadata: &Vec<NodeMetadata>,
		edges: &Vec<(usize, usize)>,
	) -> Vec<DVec2> {
		let rng = &Rng::with_seed(0);

		let mut adjacency_list = vec![Vec::new(); metadata.len()];
		for &(a, b) in edges {
			adjacency_list[a].push(b);
		}

		let mut node_depth = vec![usize::MAX; metadata.len()];
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

		(0..metadata.len())
			.map(|i| (random_dvec2(rng) + DVec2::Y) * node_depth[i] as f64)
			.collect()
	}

	pub fn run_layout_iterations(&mut self, iterations: usize, params: EmbeddingParameters) -> f64 {
		if self.node_positions.is_empty() {
			return 0.0;
		}
		let mut node_velocity = vec![DVec2::ZERO; self.node_positions.len()];
		let mut node_accel = vec![DVec2::ZERO; self.node_positions.len()];
		let rng = &Rng::new();
		let mut total_delta_pos: f64 = 0.0;

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
			// Nodes repell with `F \propto D^-2`
			if params.repulsion_approximation > 0.0 {
				let tree = BarnesHutRTree::new(&mut self.node_positions.clone());
				for i in 0..self.node_positions.len() {
					node_accel[i] += params.repulsion
						* tree.force_on(
							self.node_positions[i],
							params.repulsion_approximation,
						);
				}
			} else {
				for a in 0..self.node_positions.len() {
					for b in 0..self.node_positions.len() {
						let a_to_b = self.node_positions[b] - self.node_positions[a];
						let push = params.repulsion * a_to_b / (1.0 + a_to_b.length().powi(3));
						node_accel[a] -= push;
						node_accel[b] += push;
					}
				}
			}
			let a0 = node_accel[0];
			for i in 1..self.node_positions.len() {
				// Gravity
				node_accel[i] += DVec2::Y * params.gravity;
				node_accel[i] -= a0;
				// Opposite accel and velocity => exponentially reduce velocity
				if node_accel[i].dot(node_velocity[i]) > 0.0 {
					const VELOCITY_SPEEDUP: f64 = 1.1;
					node_velocity[i] *= VELOCITY_SPEEDUP;
				} else {
					const VELOCITY_SLOWDOWN: f64 = 0.9;
					node_velocity[i] *= VELOCITY_SLOWDOWN;
				}
				node_velocity[i] += node_accel[i];
				let delta_pos = node_velocity[i] + random_dvec2(rng) * (params.noise * temperature);
				self.node_positions[i] += delta_pos;
				total_delta_pos += delta_pos.length_squared();
				if !self.node_positions[i].is_finite() {
					tracing::warn!("infinite node position; resetting graph");
					*self = Self {
						node_positions: Self::initial_node_positions(
							&self.node_metadata,
							&self.edges,
						),
						node_metadata: mem::take(&mut self.node_metadata),
						edges: mem::take(&mut self.edges),
						min: DVec2::ZERO,
						max: DVec2::ZERO,
						gui_zoom: 1.0,
						gui_pos: emath::Vec2::ZERO,
					};
					return f64::INFINITY;
				}
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
		total_delta_pos
	}
}

fn random_dvec2(rng: &Rng) -> DVec2 {
	DVec2 {
		x: rng.f64(),
		y: rng.f64(),
	}
}

/// Barnes-Hut implemented with a 2D R-tree
enum BarnesHutRTree {
	Leaf {
		point_mass: DVec2,
	},
	Split {
		mass: f64,
		center_of_mass: DVec2,
		tight_bounding_box: [DVec2; 2],
		children: [Box<Self>; 2],
	},
}
impl BarnesHutRTree {
	fn new(positions: &mut [DVec2]) -> Self {
		assert!(positions.len() > 0);
		if positions.len() == 1 {
			return Self::Leaf {
				point_mass: positions[0],
			};
		}
		let tight_bounding_box = {
			let min = positions.iter().copied().reduce(DVec2::min).unwrap();
			let max = positions.iter().copied().reduce(DVec2::max).unwrap();
			[min, max]
		};
		let split_extractor: fn(DVec2) -> f64 = if tight_bounding_box[1].x - tight_bounding_box[0].x
			> tight_bounding_box[1].y - tight_bounding_box[0].y
		{
			|v| v.x
		} else {
			|v| v.y
		};
		positions.sort_by(|&a, &b| f64::total_cmp(&split_extractor(a), &split_extractor(b)));

		let (before, after) = positions.split_at_mut(positions.len() / 2);
		let children = [Box::new(Self::new(before)), Box::new(Self::new(after))];

		Self::Split {
			mass: positions.len() as f64,
			center_of_mass: (children[0].center_of_mass() * children[0].mass()
				+ children[1].center_of_mass() * children[1].mass())
				/ positions.len() as f64,
			tight_bounding_box,
			children,
		}
	}

	fn mass(&self) -> f64 {
		match *self {
			Self::Leaf { .. } => 1.0,
			Self::Split { mass, .. } => mass,
		}
	}

	fn center_of_mass(&self) -> DVec2 {
		match *self {
			Self::Leaf { point_mass } => point_mass,
			Self::Split { center_of_mass, .. } => center_of_mass,
		}
	}

	fn force_on(&self, pos: DVec2, approximation: f64) -> DVec2 {
		match self {
			Self::Leaf { point_mass } => Self::force_on_from_source(*point_mass, pos),
			Self::Split {
				mass,
				center_of_mass,
				tight_bounding_box,
				children,
			} => {
				let mid = (tight_bounding_box[0] + tight_bounding_box[1]) / 2.0;
				let long_side = (tight_bounding_box[1] - tight_bounding_box[0]).max_element();
				if approximation * mid.distance(pos) > long_side {
					*mass * Self::force_on_from_source(*center_of_mass, pos)
				} else {
					children[0].force_on(pos, approximation)
						+ children[1].force_on(pos, approximation)
				}
			}
		}
	}

	fn force_on_from_source(source: DVec2, target: DVec2) -> DVec2 {
		let delta = target - source;
		delta / (1.0 + delta.length().powi(3))
	}
}
