use std::{
	cmp::{Ordering, PartialEq},
	mem,
};

use fastrand::Rng;
use glam::DVec2;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::LodText;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct EmbeddingParameters {
	pub noise: f64,
	pub attraction: f64,
	pub repulsion: f64,
	pub gravity: f64,
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
			statistic_updates_per_second: 1.0,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EmbedderHasConverged {
	Yes,
	No,
}
impl EmbedderHasConverged {
	pub fn and_also(self, other: Self) -> Self {
		match self {
			Self::Yes => other,
			Self::No => Self::No,
		}
	}

	pub fn converged(self) -> bool {
		match self {
			Self::Yes => true,
			Self::No => false,
		}
	}
}

#[derive(Clone, Debug)]
pub struct NodeDrawingData {
	pub state: usize,
	pub scc_group: usize,
	pub function: usize,
	pub lod_text: LodText,
}

#[derive(Clone, Debug)]
pub struct Graph2D {
	pub(crate) node_positions: Vec<DVec2>,
	pub(crate) node_drawing_data: Vec<NodeDrawingData>,
	pub(crate) edges: Vec<(usize, usize)>,
	pub(crate) min: DVec2,
	pub(crate) max: DVec2,
	// Convergence variables
	params: EmbeddingParameters,
	repulsion_approximation: f64,
	time_step: f64,
	best_potential_energy: f64,
	iterations_since_improved_best_potential_energy: usize,
}

impl Default for Graph2D {
	fn default() -> Self {
		Graph2D::empty()
	}
}

impl Graph2D {
	const BARNES_HUT_CUTOFF: f64 = 0.1;
	const ITERATIONS_PER_LEVEL: usize = 10;
	const MAX_REPULSION_APPROXIMATION: f64 = 0.6;
	const MAX_TIME_STEP: f64 = 1.0;
	const STEP_REPULSION_APPROXIMATION: f64 = 0.1;
	const STEP_TIME_STEP: f64 = 0.1;

	pub fn empty() -> Self {
		Self {
			node_positions: Vec::new(),
			node_drawing_data: Vec::new(),
			edges: Vec::new(),
			min: DVec2::ZERO,
			max: DVec2::ZERO,
			params: EmbeddingParameters::default(),
			repulsion_approximation: Self::MAX_REPULSION_APPROXIMATION,
			time_step: Self::MAX_TIME_STEP,
			best_potential_energy: f64::INFINITY,
			iterations_since_improved_best_potential_energy: 0,
		}
	}

	pub fn set_params(&mut self, mut params: EmbeddingParameters) {
		params.statistic_updates_per_second = 1.0;
		if self.params.partial_cmp(&params).unwrap() == Ordering::Equal {
			return;
		}
		self.params = params;
		self.repulsion_approximation = Self::MAX_REPULSION_APPROXIMATION;
		self.time_step = Self::MAX_TIME_STEP;
		self.best_potential_energy = f64::INFINITY;
		self.iterations_since_improved_best_potential_energy = 0;
	}

	/// Equivalent to `*self = Graph2D::new(node_count, edges)`, but with a better
	/// initial layout guess.
	pub fn seeded_replace_self_with(
		&mut self,
		nodes: Vec<NodeDrawingData>,
		edges: Vec<(usize, usize)>,
	) {
		let num_nodes = nodes.len();
		let old = mem::replace(self, Self::new(nodes, edges));

		let shared_count = usize::min(old.node_positions.len(), num_nodes);
		self.node_positions[..shared_count].copy_from_slice(&old.node_positions[..shared_count]);

		const INITIAL_NOISE: f64 = 0.1;
		let rng = &Rng::with_seed(0);
		self.node_positions
			.iter_mut()
			.for_each(|pos| *pos += INITIAL_NOISE * random_dvec2(rng));
	}

	pub fn new(nodes: Vec<NodeDrawingData>, edges: Vec<(usize, usize)>) -> Self {
		if nodes.is_empty() {
			return Self::empty();
		}

		Self {
			node_positions: Self::initial_node_positions(nodes.len(), &edges),
			node_drawing_data: nodes,
			edges,
			..Self::empty()
		}
	}

	pub fn get_node_text(&self, node_id: usize) -> &str {
		self.node_drawing_data[node_id].lod_text.get_full()
	}

	fn initial_node_positions(node_count: usize, edges: &[(usize, usize)]) -> Vec<DVec2> {
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

	pub fn run_layout_iterations(&mut self, iterations: usize) -> EmbedderHasConverged {
		if self.node_positions.is_empty() {
			return EmbedderHasConverged::Yes;
		}
		let mut node_velocity = vec![DVec2::ZERO; self.node_positions.len()];
		let mut node_accel = vec![DVec2::ZERO; self.node_positions.len()];
		let mut tree_buffer =
			vec![BarnesHutRTree::default(); 2 * self.node_positions.len().next_power_of_two()];
		let rng = &Rng::new();
		let mut potential_energy: f64 = 0.0;

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
				// `F = k D^{1.2}`
				let push = self.params.attraction * delta * scale;
				node_accel[a] += push;
				node_accel[b] -= push;
				// `E = k D^{2.2} / 2.2
				potential_energy +=
					self.params.attraction * delta.length_squared() * scale / (2.0 + scale);
			}
			// Nodes repell with `F \propto D^-2`
			if self.repulsion_approximation > Self::BARNES_HUT_CUTOFF {
				BarnesHutRTree::build_in(&mut tree_buffer, &mut self.node_positions.clone());
				potential_energy += node_accel
					.par_iter_mut()
					.zip_eq(&self.node_positions)
					.map(|(accel, &pos)| {
						let mut pot_en = 0.0;
						*accel += self.params.repulsion
							* BarnesHutRTree::force_on(
								pos,
								&tree_buffer,
								self.repulsion_approximation.powi(2),
								&mut pot_en,
							);
						pot_en
					})
					.sum::<f64>();
			} else {
				potential_energy += node_accel
					.par_iter_mut()
					.zip_eq(&self.node_positions)
					.map(|(a_accel, &a_pos)| {
						let mut pot_en = 0.0;
						for &b_pos in &self.node_positions {
							let a_to_b = b_pos - a_pos;
							let a_to_b_len = a_to_b.length();
							// `F = k D / (1 + D^3)`
							let push = self.params.repulsion * a_to_b / (1.0 + a_to_b_len.powi(3));
							*a_accel -= push;
							// `E = k / (1 + D)`
							pot_en += self.params.repulsion / (1.0 + a_to_b_len);
						}
						pot_en
					})
					.sum::<f64>();
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
				accel += DVec2::Y * self.params.gravity;
				accel -= a0;
				potential_energy -= self.params.gravity * pos.y;
				// Opposite accel and velocity => exponentially reduce velocity
				if accel.dot(*vel) > 0.0 {
					const VELOCITY_SPEEDUP: f64 = 1.1;
					*vel *= VELOCITY_SPEEDUP;
				} else {
					const VELOCITY_SLOWDOWN: f64 = 0.9;
					*vel *= VELOCITY_SLOWDOWN;
				}
				*vel += accel * self.time_step;
				let delta_pos =
					*vel * self.time_step + random_dvec2(rng) * (self.params.noise * temperature);
				*pos += delta_pos;
				if !pos.is_finite() {
					tracing::warn!("infinite node position; resetting graph");
					*self = Self {
						node_positions: Self::initial_node_positions(
							self.node_positions.len(),
							&self.edges,
						),
						node_drawing_data: mem::take(&mut self.node_drawing_data),
						edges: mem::take(&mut self.edges),
						..Self::empty()
					};
					return EmbedderHasConverged::No;
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

		if self.params.noise > 0.0 {
			self.best_potential_energy = potential_energy;
			return EmbedderHasConverged::No;
		}

		if potential_energy < self.best_potential_energy {
			self.best_potential_energy = potential_energy;
			self.iterations_since_improved_best_potential_energy = 0;
		} else {
			self.iterations_since_improved_best_potential_energy += 1;
			if self.iterations_since_improved_best_potential_energy > 0
				&& self.iterations_since_improved_best_potential_energy % Self::ITERATIONS_PER_LEVEL
					== 0
			{
				self.repulsion_approximation = (self.repulsion_approximation
					- Self::STEP_REPULSION_APPROXIMATION)
					.clamp(0.0, Self::MAX_REPULSION_APPROXIMATION);
				if self.repulsion_approximation == 0.0 {
					self.time_step = (self.time_step / 2.0 - Self::STEP_TIME_STEP)
						.clamp(0.0, Self::MAX_TIME_STEP);
				}
			}
		}

		if self.time_step == 0.0 {
			EmbedderHasConverged::Yes
		} else {
			EmbedderHasConverged::No
		}
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

	fn force_on(
		pos: DVec2,
		tree: &[Self],
		approximation2: f64,
		potential_energy: &mut f64,
	) -> DVec2 {
		match tree[0] {
			Self::Leaf { point_mass } => {
				Self::force_on_from_source(1.0, point_mass, pos, potential_energy)
			}
			Self::Split {
				mass,
				center_of_mass,
				tight_bounding_box,
			} => {
				let mid = (tight_bounding_box[0] + tight_bounding_box[1]) / 2.0;
				let long_side = (tight_bounding_box[1] - tight_bounding_box[0]).max_element();
				if approximation2 * mid.distance_squared(pos) > long_side * long_side {
					Self::force_on_from_source(mass, center_of_mass, pos, potential_energy)
				} else {
					let (_, tree_children) = tree.split_at(1);
					let (tree_left, tree_right) = tree_children.split_at(tree_children.len() / 2);
					Self::force_on(pos, tree_left, approximation2, potential_energy)
						+ Self::force_on(pos, tree_right, approximation2, potential_energy)
				}
			}
		}
	}

	fn force_on_from_source(
		mass: f64,
		source: DVec2,
		target: DVec2,
		potential_energy: &mut f64,
	) -> DVec2 {
		let delta = target - source;
		let delta_len = delta.length();
		let force = mass * delta / (1.0 + delta_len.powi(3));
		*potential_energy += mass / (1.0 + delta_len);
		force
	}
}
