use std::mem;

use fastrand::Rng;
use glam::DVec2;
use ipc::GraphIpc;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

#[derive(Clone, Debug)]
pub struct Graph2D {
	pub node_positions: Vec<DVec2>,
	pub edges: Vec<(usize, usize)>,
	pub min: DVec2,
	pub max: DVec2,
	pub gui_zoom: f32,
	pub gui_pos: emath::Vec2,
}

impl From<GraphIpc> for Graph2D {
	fn from(graph: GraphIpc) -> Self {
		if graph.metadata.is_empty() {
			return Self::empty();
		}

		Self {
			node_positions: Self::initial_node_positions(graph.metadata.len(), &graph.edges),
			edges: graph.edges,
			min: DVec2::ZERO,
			max: DVec2::ZERO,
			gui_zoom: 1.0,
			gui_pos: emath::Vec2::ZERO,
		}
	}
}

impl Default for Graph2D {
	fn default() -> Self {
		Graph2D::empty()
	}
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
			node_positions: Vec::new(),
			edges: Vec::new(),
			min: DVec2::ZERO,
			max: DVec2::ZERO,
			gui_zoom: 1.0,
			gui_pos: emath::Vec2::ZERO,
		}
	}

	pub fn from_old(old: Self, graph: impl Into<GraphIpc>, params: EmbeddingParameters) -> Self {
		let mut new = Graph2D::new(graph, params);
		for (from, to) in old.node_positions.iter().zip(new.node_positions.iter_mut()) {
			*to = *from;
		}
		new
	}

	/// Create a new `Graph2D` with as many node positions as possible stolen from the old graph.
	/// Not perfect by any means, but a cheap hack to minimise jumpiness during updates
	pub fn into_new(&mut self, graph: impl Into<GraphIpc>, params: EmbeddingParameters) {
		let old = mem::take(self);
		let new = Graph2D::from_old(old, graph, params);
		*self = new;
	}

	pub fn new(graph: impl Into<GraphIpc>, params: EmbeddingParameters) -> Self {
		let graph_: GraphIpc = graph.into();
		let mut ret: Graph2D = graph_.into();
		ret.run_layout_iterations(100, params);
		ret
	}

	fn initial_node_positions(node_count: usize, edges: &Vec<(usize, usize)>) -> Vec<DVec2> {
		let rng = &Rng::with_seed(0);

		let mut adjacency_list = vec![Vec::new(); node_count];
		for &(a, b) in edges {
			adjacency_list[a].push(b);
		}

		let mut node_depth = vec![usize::MAX; node_count];
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

		node_depth
			.into_iter()
			.map(|d| (random_dvec2(rng) + DVec2::Y) * d as f64)
			.collect()
	}

	pub fn run_layout_iterations(&mut self, iterations: usize, params: EmbeddingParameters) -> f64 {
		if self.node_positions.is_empty() {
			return 0.0;
		}
		let mut node_velocity = vec![DVec2::ZERO; self.node_positions.len()];
		let mut node_accel = vec![DVec2::ZERO; self.node_positions.len()];
		let mut tree_buffer =
			vec![BarnesHutRTree::default(); 2 * self.node_positions.len().next_power_of_two()];
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
				BarnesHutRTree::build_in(&mut tree_buffer, &mut self.node_positions.clone());
				node_accel
					.par_iter_mut()
					.zip_eq(&self.node_positions)
					.for_each(|(accel, &pos)| {
						*accel += params.repulsion
							* BarnesHutRTree::force_on(
								pos,
								&tree_buffer,
								params.repulsion_approximation.powi(2),
							);
					});
			} else {
				node_accel
					.par_iter_mut()
					.zip_eq(&self.node_positions)
					.for_each(|(a_accel, &a_pos)| {
						for &b_pos in &self.node_positions {
							let a_to_b = b_pos - a_pos;
							let push = params.repulsion * a_to_b / (1.0 + a_to_b.length().powi(3));
							*a_accel -= push;
						}
					});
			}
			let a0 = node_accel[0];
			for ((pos, vel), &(mut accel)) in self
				.node_positions
				.iter_mut()
				.zip(&mut node_velocity)
				.zip(&node_accel)
				.skip(1)
			{
				// Gravity
				accel += DVec2::Y * params.gravity;
				accel -= a0;
				// Opposite accel and velocity => exponentially reduce velocity
				if accel.dot(*vel) > 0.0 {
					const VELOCITY_SPEEDUP: f64 = 1.1;
					*vel *= VELOCITY_SPEEDUP;
				} else {
					const VELOCITY_SLOWDOWN: f64 = 0.9;
					*vel *= VELOCITY_SLOWDOWN;
				}
				*vel += accel;
				let delta_pos = *vel + random_dvec2(rng) * (params.noise * temperature);
				*pos += delta_pos;
				total_delta_pos += delta_pos.length_squared();
				if !pos.is_finite() {
					tracing::warn!("infinite node position; resetting graph");
					*self = Self {
						node_positions: Self::initial_node_positions(
							self.node_positions.len(),
							&self.edges,
						),
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
			for (pos, vel) in Iterator::zip(
				self.node_positions.iter_mut(),
				node_velocity.iter_mut(),
			)
			.skip(1)
			{
				*pos = rotate_down_center_of_mass.rotate(*pos);
				*vel = rotate_down_center_of_mass.rotate(*vel);
			}
		}
		(self.min, self.max) = self
			.node_positions
			.iter()
			.fold((DVec2::ZERO, DVec2::ZERO), |(min, max), &val| {
				(min.min(val), max.max(val))
			});
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
#[derive(Clone, Copy)]
enum BarnesHutRTree {
	Leaf {
		point_mass: DVec2,
	},
	Split {
		mass: f64,
		center_of_mass: DVec2,
		tight_bounding_box: [DVec2; 2],
	},
}

impl Default for BarnesHutRTree {
	fn default() -> Self {
		Self::Leaf {
			point_mass: DVec2::ZERO,
		}
	}
}

impl BarnesHutRTree {
	fn build_in(buf: &mut [Self], positions: &mut [DVec2]) {
		assert!(!positions.is_empty());
		assert!(!buf.is_empty());
		if positions.len() == 1 {
			buf[0] = Self::Leaf {
				point_mass: positions[0],
			};
			return;
		}
		let tight_bounding_box = positions.iter().fold(
			[DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)],
			|[min, max], &pos| [min.min(pos), max.max(pos)],
		);
		let split_extractor: fn(DVec2) -> f64 = if tight_bounding_box[1].x - tight_bounding_box[0].x
			> tight_bounding_box[1].y - tight_bounding_box[0].y
		{
			|v| v.x
		} else {
			|v| v.y
		};
		// Partition by median along the `split_extractor`:s direction using quickselect.
		let len = positions.len();
		positions.select_nth_unstable_by(len / 2, |&a, &b| {
			f64::total_cmp(&split_extractor(a), &split_extractor(b))
		});

		let (buf_self, buf_children) = buf.split_at_mut(1);
		let (buf_left, buf_right) = buf_children.split_at_mut(buf_children.len() / 2);

		let (before, after) = positions.split_at_mut(len / 2);
		const FORK_JOIN_THRESHOLD: usize = 20;
		if len > FORK_JOIN_THRESHOLD {
			rayon::join(
				|| Self::build_in(buf_left, before),
				|| Self::build_in(buf_right, after),
			);
		} else {
			Self::build_in(buf_left, before);
			Self::build_in(buf_right, after);
		}

		let center_of_mass = (buf_left[0].center_of_mass() * buf_left[0].mass()
			+ buf_right[0].center_of_mass() * buf_right[0].mass())
			/ positions.len() as f64;
		buf_self[0] = Self::Split {
			mass: positions.len() as f64,
			center_of_mass,
			tight_bounding_box,
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

	fn force_on(pos: DVec2, tree: &[Self], approximation2: f64) -> DVec2 {
		match tree[0] {
			Self::Leaf { point_mass } => Self::force_on_from_source(point_mass, pos),
			Self::Split {
				mass,
				center_of_mass,
				tight_bounding_box,
			} => {
				let mid = (tight_bounding_box[0] + tight_bounding_box[1]) / 2.0;
				let long_side = (tight_bounding_box[1] - tight_bounding_box[0]).max_element();
				if approximation2 * mid.distance_squared(pos) > long_side * long_side {
					mass * Self::force_on_from_source(center_of_mass, pos)
				} else {
					let (_, tree_children) = tree.split_at(1);
					let (tree_left, tree_right) = tree_children.split_at(tree_children.len() / 2);
					Self::force_on(pos, tree_left, approximation2)
						+ Self::force_on(pos, tree_right, approximation2)
				}
			}
		}
	}

	fn force_on_from_source(source: DVec2, target: DVec2) -> DVec2 {
		let delta = target - source;
		delta / (1.0 + delta.length().powi(3))
	}
}
