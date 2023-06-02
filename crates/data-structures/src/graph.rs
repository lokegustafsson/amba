use std::{
	collections::{BTreeSet, HashMap},
	default::Default,
	mem,
};

use smallvec::smallvec;

use crate::small_set::SmallU64Set;
type SmallVec<T> = smallvec::SmallVec<[T; crate::small_set::SMALL_SIZE]>;

// Aliased so we can swap them to BTree versions easily.
pub(crate) type Set<T> = std::collections::BTreeSet<T>;
pub(crate) type Map<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Debug, Clone, Default)]
pub struct Node {
	pub id: u64,
	pub from: SmallU64Set,
	pub to: SmallU64Set,
	pub of: SmallVec<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
	pub nodes: Map<u64, Node>,
	pub merges: Map<u64, u64>,
}

impl Graph {
	pub fn new() -> Self {
		Default::default()
	}

	#[cfg(test)]
	pub fn with_nodes(nodes: Map<u64, Node>) -> Self {
		Graph {
			nodes,
			merges: Map::new(),
		}
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	pub fn get(&mut self, mut idx: u64) -> Option<&Node> {
		idx = translate(idx, &mut self.merges);
		self.nodes.get(&idx)
	}

	pub fn get_mut(&mut self, mut idx: u64) -> Option<&mut Node> {
		idx = translate(idx, &mut self.merges);
		self.nodes.get_mut(&idx)
	}

	pub fn apply_merges(&mut self) {
		for Node { from, to, .. } in self.nodes.values_mut() {
			*from = from
				.iter()
				.copied()
				.map(|x| translate(x, &mut self.merges))
				.collect();
			*to = to
				.iter()
				.copied()
				.map(|x| translate(x, &mut self.merges))
				.collect();
		}
		self.merges = Map::new();
	}

	/// Insert a node connection. Returns true if the connection
	/// is new.
	pub fn update(&mut self, from: u64, to: u64) -> bool {
		let mut modified = false;
		self.nodes
			.entry(from)
			.and_modify(|node| {
				modified |= node.to.insert(to);
			})
			.or_insert_with(|| {
				modified = true;
				let to = [to].into_iter().collect::<SmallU64Set>();
				let of: SmallVec<u64> = smallvec![from];
				Node {
					id: from,
					to,
					from: Default::default(),
					of,
				}
			});
		self.nodes
			.entry(to)
			.and_modify(|node| {
				modified |= node.from.insert(from);
			})
			.or_insert_with(|| {
				modified = true;
				let from = [from].into_iter().collect::<SmallU64Set>();
				let of: SmallVec<u64> = smallvec![to];
				Node {
					id: to,
					to: Default::default(),
					from,
					of,
				}
			});

		if self.is_loop(from) {
			self.rotate_of(from);
		}

		if self.is_loop(to) {
			self.rotate_of(to);
		}
		modified
	}

	// Returns true if the given node has an edge to itself
	pub fn is_loop(&mut self, node: u64) -> bool {
		if let Some(node) = self.get(node) {
			return node.to.contains(&node.id)
				&& node.from.contains(&node.id)
				&& node.to.len() == 1
				&& node.from.len() == 1;
		} else {
			return false;
		}
	}

	// Rotates of until least id is first
	// for given node
	pub fn rotate_of(&mut self, node: u64) {
		if let Some(node) = self.nodes.get_mut(&node) {
			if node.of.iter().len() > 1 {
				let min_index = node
					.of
					.iter()
					.position(|&x| x == *node.of.iter().min().unwrap())
					.unwrap();
				node.of.rotate_left(min_index);
			}
		}
	}

	// Updates compressed graph with new connection
	// Splits compressed nodes if the connection:
	// 1. Adds incoming edge to a non-first node
	// 2. Adds outgoing edge to a non-last node
	pub fn revert_and_update(&mut self, from: u64, to: u64) {
		let mut from_node = from;
		let mut to_node = to;
		let rest;

		// Finds the from node in the graph, returns none if not yet added
		fn find_from(graph: &Graph, from_node: u64) -> Option<&Node> {
			if let Some(node) = graph.nodes.get(&from_node) {
				return Some(node);
			} else {
				for node in graph.nodes.values() {
					if node.of.iter().any(|&x| x == from_node) {
						return Some(node);
					}
				}
				return None;
			}
		}

		// Finds the to node in the graph, returns none if not yet added
		fn find_to(graph: &Graph, to_node: u64) -> Option<&Node> {
			if let Some(node) = graph.nodes.get(&to_node) {
				return Some(node);
			} else {
				for node in graph.nodes.values() {
					if node.of.iter().any(|&x| x == to_node) {
						return Some(node);
					}
				}
				return None;
			}
		}

		// Returns true if given node and to are in the same compressed node
		fn in_same(node: &Node, to: u64) -> bool {
			return node.of.iter().any(|&x| x == to);
		}

		// Self loop
		if from == to {
			if let Some(node) = find_from(self, from) {
				if node.id == from && node.of.iter().len() == 1 {
					// no need to split any compressed nodes
					self.update(from, from);
					return;
				} else {
					// self loop in a compressed node, split around it
					(from_node, _) = self.split_after(from, node.id);
					(_, to_node) = self.split_before(to, from_node);
					self.update(to_node, to_node);
					return;
				}
			}
		}

		// find from node
		if let Some(node) = find_from(self, from) {
			// from node exists
			let of = node.of.clone();

			// is to node in same node?
			let same = in_same(node, to);

			// to-node is directly after from-node, nothing to do
			if same
				&& (of.iter().position(|&x| x == to).unwrap()
					== of.iter().position(|&x| x == from).unwrap() + 1)
			{
				return;
			}

			(from_node, rest) = self.split_after(from, node.id);

			// to was somewhere else in same
			if same {
				if in_same(self.nodes.get(&from_node).unwrap(), to) {
					// to node still in same after first split
					(_, to_node) = self.split_before(to, from_node);
					self.update(to_node, to_node);
					return;
				} else {
					// to node ended up in the other partition after first split
					(_, to_node) = self.split_before(to, rest);
					self.update(from_node, to_node);
					return;
				}
			}
		}

		// to node wasn't found in same as from node
		if let Some(node) = find_to(self, to) {
			(_, to_node) = self.split_before(to, node.id);
		}
		self.update(from_node, to_node);
	}

	/// Split a merged node after occurence of "after" node
	/// returns the node id's of new nodes after split
	pub fn split_after(&mut self, after: u64, merged_in: u64) -> (u64, u64) {
		let node = self.get(merged_in).unwrap().clone();
		let mut of = node.of.clone();

		if self.is_loop(merged_in) {
			if let Some(index) = of.iter().position(|&x| x == after) {
				let rotate = of.len() - index - 1;
				self.get_mut(merged_in).unwrap().of.rotate_right(rotate);
				of = self.get(merged_in).unwrap().of.clone();
			}
		}

		for (i, &subnode) in of.iter().enumerate() {
			if subnode == after {
				if i < of.len() - 1 {
					let part1_of = of.get(..i + 1).unwrap();
					let part2_of = of.get(i + 1..).unwrap();

					let part1 = part1_of.iter().min().unwrap();
					let part2 = part2_of.iter().min().unwrap();

					let partition_1 = Node {
						id: *part1,
						from: node.from.clone(),
						to: [*part2].into_iter().collect::<SmallU64Set>(),
						of: part1_of.into(),
					};
					let partition_2 = Node {
						id: *part2,
						from: [*part1].into_iter().collect::<SmallU64Set>(),
						to: node.to.clone(),
						of: part2_of.into(),
					};
					self.merges.remove(part2);
					self.merges.remove(part1);

					self.nodes.remove(&merged_in);
					self.nodes.insert(*part1, partition_1);
					self.nodes.insert(*part2, partition_2);

					self.update_from_after_split(merged_in, *part2, node.to);
					self.update_to_after_split(merged_in, *part1, node.from);

					return (*part1, *part2);
				} else {
					return (node.id, node.id);
				}
			}
		}
		(after, after)
	}

	/// Split merged node before occurrence of new "to" connection
	/// returns the node id's of new nodes after split
	pub fn split_before(&mut self, before: u64, merged_in: u64) -> (u64, u64) {
		let node = self.get(merged_in).unwrap().clone();
		let of = node.of.clone();

		for (i, &subnode) in of.iter().enumerate() {
			if subnode == before {
				if i > 0 {
					let part1_of = of.get(..i).unwrap();
					let part2_of = of.get(i..).unwrap();

					let part1 = part1_of.iter().min().unwrap();
					let part2 = part2_of.iter().min().unwrap();

					let partition_1 = Node {
						id: *part1,
						from: node.from.clone(),
						to: [*part2].into_iter().collect::<SmallU64Set>(),
						of: part1_of.into(),
					};
					let partition_2 = Node {
						id: *part2,
						from: [*part1].into_iter().collect::<SmallU64Set>(),
						to: node.to.clone(),
						of: part2_of.into(),
					};
					self.merges.remove(part2);
					self.merges.remove(part1);

					self.nodes.remove(&merged_in);
					self.nodes.insert(*part1, partition_1);
					self.nodes.insert(*part2, partition_2);

					self.update_from_after_split(merged_in, *part2, node.to);
					self.update_to_after_split(merged_in, *part1, node.from);

					return (*part1, *part2);
				} else {
					return (node.id, node.id);
				}
			}
		}
		(before, before)
	}

	/// after a split, update other nodes "from" values to new node id
	fn update_from_after_split(&mut self, old: u64, new: u64, nodes: SmallU64Set) {
		for node in nodes {
			if let Some(to_update) = self.nodes.get_mut(&node) {
				if to_update.from.remove(&old) {
					to_update.from.insert(new);
				}
			}
		}
	}

	/// after a split, update other nodes "to" value to new node id
	fn update_to_after_split(&mut self, old: u64, new: u64, nodes: SmallU64Set) {
		for node in nodes {
			if let Some(to_update) = self.nodes.get_mut(&node) {
				if to_update.to.remove(&old) {
					to_update.to.insert(new);
				}
			}
		}
	}

	/// Compresses graph by merging every node pair that always go
	/// from one to the other
	pub fn compress(&mut self) {
		let m = &mut self.nodes;

		// Visit every node in arbitrary order.
		// We have to check (a, b) AND (b, a) seperately
		//  because we have a directed *cyclic* graph.
		// Following a depth-first order would just require
		//  a visited collection for no benefit.
		// We have to traverse the graph twice anyway because
		//  of the borrow checker.
		// The merged node will take the id of the smallest of
		//  the parents.

		let to_merge = m
			.values()
			.filter(|l| l.to.len() == 1)
			.map(|l| {
				let key = translate(l.to.get_any(), &mut self.merges);
				(l, &m[&key])
			})
			.filter(|(_, r)| r.from.len() == 1)
			.map(|(l, r)| (l.id, r.id))
			.collect::<Set<_>>();

		for (mut l, mut r) in to_merge.into_iter() {
			l = translate(l, &mut self.merges);
			r = translate(r, &mut self.merges);

			//let x = l.min(r);
			//let y = l.max(r);
			self.merge_nodes(l, r);
		}
	}

	/// Compress around given candidates. If a candidate gets
	/// compressed its neighbours will be checked too, growing out
	/// from there.
	//pub fn compress_with_hint(&mut self, nodes: SmallU64Set) {
	pub fn compress_with_hint(&mut self, mut from: u64, mut to: u64) {
		from = translate(from, &mut self.merges);
		to = translate(to, &mut self.merges);
		// The queue is a set so we can guarantee that there are no
		// duplicates in the queue and HashSet doesn't have a pop
		// function.
		let mut candidates = [(from, to)].into_iter().collect::<BTreeSet<_>>();
		for pair in self.get(from).unwrap().from.iter().map(|&f| (f, from)) {
			candidates.insert(pair);
		}
		for pair in self.get(from).unwrap().to.iter().map(|&t| (from, t)) {
			candidates.insert(pair);
		}
		for pair in self.get(to).unwrap().to.iter().map(|&t| (to, t)) {
			candidates.insert(pair);
		}
		for pair in self.get(to).unwrap().from.iter().map(|&f| (f, to)) {
			candidates.insert(pair);
		}
		self.compress_with_hint_2(candidates);
	}

	fn compress_with_hint_2(&mut self, mut queued: BTreeSet<(u64, u64)>) {
		while let Some((mut from, mut to)) = queued.pop_first() {
			from = translate(from, &mut self.merges);
			to = translate(to, &mut self.merges);

			if !self.are_mergable_link(from, to) {
				continue;
			}
			let this = self.merge_nodes(from, to);
			let node = &self.nodes[&this];

			let tos = node
				.to
				.iter()
				.filter(|&&n| n != this)
				.copied()
				.map(move |t| (this, t));
			let froms = node
				.from
				.iter()
				.filter(|&&n| n != this)
				.copied()
				.map(move |f| (f, this));
			for connection in tos.chain(froms) {
				queued.insert(connection);
			}
		}
	}

	fn are_mergable_link(&mut self, mut l: u64, mut r: u64) -> bool {
		l = translate(l, &mut self.merges);
		r = translate(r, &mut self.merges);

		let mut f = |x, y| {
			if self.nodes[&x].to.len() != 1 || self.nodes[&y].from.len() != 1 {
				return false;
			}

			let to_link = self.nodes[&x].to.get_any();
			let from_link = self.nodes[&y].from.get_any();

			translate(to_link, &mut self.merges) == y && translate(from_link, &mut self.merges) == x
		};

		f(l, r)
	}

	fn are_loop(&mut self, l: u64, r: u64) -> bool {
		if l == r {
			return true;
		}
		let m = &self.nodes;

		let there = m[&l]
			.from
			.iter()
			.map(|&i| translate(i, &mut self.merges))
			.any(|x| x == r);
		let back_again = m[&r]
			.from
			.iter()
			.map(|&i| translate(i, &mut self.merges))
			.any(|x| x == l);

		there && back_again
	}

	/// Returns the id of the merged node
	pub fn merge_nodes(&mut self, l: u64, r: u64) -> u64 {
		if l == r {
			return l;
		}

		assert!(self.nodes.contains_key(&l));
		assert!(self.nodes.contains_key(&r));

		let are_loop = self.are_loop(l, r);
		let map = &mut self.nodes;

		// Must be after are_loop
		self.merges.insert(r.max(l), r.min(l));
		self.merges.remove(&r.min(l));

		let combine_sets = |mut left_set: SmallU64Set, mut right_set: SmallU64Set| -> SmallU64Set {
			if left_set.len() < right_set.len() {
				mem::swap(&mut left_set, &mut right_set);
			}
			for node in right_set.into_iter() {
				left_set.insert(node);
			}

			left_set
		};

		// Take the union of both nodes' input and then remove
		// the nodes themselves
		let r_ = map.get_mut(&r.max(l)).unwrap();
		let to_r = mem::take(&mut r_.to);
		let from_r = mem::take(&mut r_.from);
		let mut of_r = mem::take(&mut r_.of);

		let l_ = map.get_mut(&r.min(l)).unwrap();
		let to_l = mem::take(&mut l_.to);
		let from_l = mem::take(&mut l_.from);
		let mut of_l = mem::take(&mut l_.of);

		l_.to = combine_sets(to_l, to_r);
		l_.from = combine_sets(from_l, from_r);

		if l > r {
			l_.of.append(&mut of_r);
			l_.of.append(&mut of_l);
		} else {
			l_.of.append(&mut of_l);
			l_.of.append(&mut of_r);
		}

		for parent in l_.of.iter() {
			l_.to.remove(parent);
			l_.from.remove(parent);
		}

		let mut l__ = l_.clone();

		let old = r.max(l);
		let new = r.min(l);

		// Restore loop if they were a loop beforehand
		if are_loop {
			l_.from.insert(new);
			l_.to.insert(new);
			l__ = l_.clone();
		}

		// Remove the node with higher id from the graph
		map.remove(&r.max(l));

		if self.is_loop(new) {
			self.rotate_of(new);
		}

		let tos = (l__.to).clone();
		let froms = (l__.from).clone();

		self.update_from_after_split(old, new, tos);
		self.update_to_after_split(old, new, froms);
		//l_.of.insert(l);
		//l_.of.insert(r);

		new
	}

	pub fn edges(&self) -> impl '_ + Iterator<Item = (u64, u64)> {
		self.nodes
			.values()
			.flat_map(|n| n.to.iter().map(|&m| (n.id, m)))
	}

	pub fn edge_list_sequentially_renamed(&self) -> Vec<(usize, usize)> {
		// NOTE: Iterating in increasing-id-order over `self.nodes` is crucial for
		// correctness (here guaranteed by BTreeMap).
		let node_id_renamings = self
			.nodes
			.keys()
			.copied()
			.enumerate()
			.map(|(x, y)| (y, x))
			.collect::<HashMap<_, _>>();
		self.edges()
			.map(|(from, to)| (node_id_renamings[&from], node_id_renamings[&to]))
			.collect()
	}

	/// Find strongly connected components in a graph. Return them as a map of original id to new nodes
	fn tarjan(&self) -> Map<u64, Node> {
		/// Associated metadata for each node
		#[derive(Copy, Clone, PartialEq, Eq, Default)]
		struct Translation {
			// A separate index from the node id, given in order nodes are found
			index: u64,
			lowest_index_child: u64,
			on_stack: bool,
		}

		#[derive(Clone, Default)]
		struct State {
			stack: Vec<u64>,
			search_metadata: Map<u64, Translation>,
			index: u64,
			out: Map<u64, Node>,
		}

		let mut state = State::default();

		fn strong_connect(graph: &Graph, state: &mut State, v: u64) {
			state.search_metadata.insert(
				v,
				Translation {
					index: state.index,
					lowest_index_child: state.index,
					on_stack: true,
				},
			);
			state.index += 1;
			state.stack.push(v);

			for &w in graph.nodes.get(&v).unwrap().to.iter() {
				match state.search_metadata.get(&w) {
					None => {
						strong_connect(graph, state, w);
						let w_low = state.search_metadata[&w].lowest_index_child;
						let v_ref = &mut state
							.search_metadata
							.get_mut(&v)
							.unwrap()
							.lowest_index_child;
						*v_ref = (*v_ref).min(w_low);
					}
					Some(Translation { on_stack, .. }) if *on_stack => {
						let w_idx = state.search_metadata[&w].index;
						let v_ref = &mut state
							.search_metadata
							.get_mut(&v)
							.unwrap()
							.lowest_index_child;
						*v_ref = (*v_ref).min(w_idx);
					}
					_ => {}
				}
			}

			let v_ref = state.search_metadata[&v];
			if v_ref.index == v_ref.lowest_index_child {
				let mut new_node = Node {
					id: v,
					..Default::default()
				};
				while let Some(w) = state.stack.pop() {
					state.search_metadata.get_mut(&w).unwrap().on_stack = false;

					let old_node = &graph.nodes[&w];
					new_node.of.push(w);
					new_node.from.union(&old_node.from);
					new_node.to.union(&old_node.to);
					new_node.id = new_node.id.min(old_node.id);

					if v == w {
						break;
					}
				}
				new_node.of.sort_unstable();
				state.out.insert(v, new_node);
			}
		}

		for node in self.nodes.values() {
			if !state.search_metadata.contains_key(&node.id) {
				strong_connect(self, &mut state, node.id);
			}
		}

		state.out
	}

	pub fn inverse_tarjan(&self) -> Map<usize, usize> {
		self.tarjan()
			.into_values()
			.flat_map(|Node { id, of, .. }| of.into_iter().map(move |o| (o as usize, id as usize)))
			.collect()
	}

	/// Returns a new graph of strongly connected components using
	/// [Tarjan's strongly connected components algorithm](https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm)
	pub fn to_strongly_connected_components_tarjan(&self) -> Self {
		let edges = self.edges().count();
		let nodes = self.len();

		// Tarjan often overflows the default stack, so if the
		// graph is large enough to cause issues, do the work
		// in a worker thread with a guaranteed large enough
		// stack
		if edges > 10_000 {
			// Keeping no variables in registers should
			// give tarjan a frame of 320 bytes. Grow by
			// the worst of nodes and edges and add 10%
			// for safety.

			let stack_size = (edges.max(nodes) as f64 * 320. * 1.1) as usize;
			let graph = self.clone();

			std::thread::Builder::new()
				.name("Tarjan worker thread".into())
				.stack_size(stack_size)
				.spawn(move || connect_dag(graph.tarjan()))
				.expect("Failed to spawn thread for tarjan")
				.join()
				.unwrap()
		} else {
			let scc = self.tarjan();
			connect_dag(scc)
		}
	}

	// Find strongly connected components in a graph. Return them as a map of original id to new nodes
	fn kosaraju(&self) -> Map<u64, Node> {
		let mut l = Vec::new(); // Backwards compared to wikipedia
		let mut visited = Set::new();
		let mut assigned = Set::new();
		let mut acc = Map::new();

		fn visit(graph: &Graph, visited: &mut Set<u64>, l: &mut Vec<u64>, u: u64) {
			if !visited.insert(u) {
				return;
			}
			for &v in graph.nodes.get(&u).unwrap().to.iter() {
				visit(graph, visited, l, v);
			}
			l.push(u);
		}
		fn assign(
			graph: &Graph,
			acc: &mut Map<u64, Node>,
			assigned: &mut Set<u64>,
			u: u64,
			root: u64,
		) {
			if !assigned.insert(u) {
				return;
			}
			let u_ref = graph.nodes.get(&u).unwrap();
			let mut ofcopy = u_ref.of.clone();
			let node = acc
				.entry(root)
				.and_modify(|Node { to, from, of, id }| {
					of.append(&mut ofcopy);
					to.union(&u_ref.to);
					from.union(&u_ref.from);
					*id = (*id).min(u);
				})
				.or_insert_with(|| u_ref.clone());
			// Because borrow checker
			let from = node.from.clone();
			for v in from.into_iter() {
				assign(graph, acc, assigned, v, root);
			}
		}

		for &u in self.nodes.keys() {
			visit(self, &mut visited, &mut l, u);
		}
		std::mem::drop(visited);

		for u in l.into_iter().rev() {
			assign(self, &mut acc, &mut assigned, u, u);
		}

		let keys = acc.keys().cloned().collect::<Vec<_>>(); //clone();
		for key in keys {
			let res = acc.get_mut(&key);
			match res {
				Some(node) => {
					node.of.sort_unstable();
				}
				None => {}
			}
		}

		acc
	}

	/// Returns a new graph of strongly connected components using
	/// [Kosaraju's Algorithm](https://en.wikipedia.org/wiki/Kosaraju%27s_algorithm)
	pub fn to_strongly_connected_components_kosaraju(&self) -> Self {
		let edges = self.edges().count();
		let nodes = self.len();

		// Kosaraju often overflows the default stack, so if the
		// graph is large enough to cause issues, do the work
		// in a worker thread with a guaranteed large enough
		// stack
		if edges > 10_000 {
			// `assign` keeps at worst three pointers and a
			// SmallU64set on the stack. Scale by the
			// worst of nodes and edges and then add 10%
			let stack_size = (nodes.max(edges) as f64 * 80. * 1.1) as usize;
			let graph = self.clone();

			std::thread::Builder::new()
				.name("Kosaraju worker thread".into())
				.stack_size(stack_size)
				.spawn(move || connect_dag(graph.kosaraju()))
				.expect("Failed to spawn thread for kosaraju")
				.join()
				.unwrap()
		} else {
			let scc = self.kosaraju();
			connect_dag(scc)
		}
	}

	/// Verify that all node pairs have matching to and from
	#[cfg(test)]
	fn verify(&self) {
		let m = &self.nodes;

		for (k, v) in m.iter() {
			for out in v.from.iter() {
				assert!(
					m[out].to.contains(k),
					"{out}.to contains {k}?\n{self:#?}"
				);
			}
		}

		for (k, v) in m.iter() {
			for to in v.to.iter() {
				assert!(
					m[to].from.contains(k),
					"{to}.from contains {k}?\n{self:#?}"
				);
			}
		}
	}
}

fn connect_dag(strongly_connected_components: Map<u64, Node>) -> Graph {
	let new_ids = strongly_connected_components
		.values()
		.flat_map(|Node { id, of, .. }| of.iter().map(|x| (*x, *id)))
		.collect::<Map<_, _>>();

	let out = strongly_connected_components
		.into_values()
		.map(|Node { id, from, to, of }| {
			let from = from
				.iter()
				.map(|x| new_ids[x])
				.filter(|&x| x != id)
				.collect();
			let to = to.iter().map(|x| new_ids[x]).filter(|&x| x != id).collect();
			(id, Node { id, from, to, of })
		})
		.collect();

	Graph {
		nodes: out,
		..Default::default()
	}
}

// Seems linear, but in practice, unless the entire
// world is one long straight line ends up never
// taking more than a single step.
fn translate(key: u64, map: &mut Map<u64, u64>) -> u64 {
	match map.get(&key) {
		Some(k) => {
			let res = translate(*k, map);
			*map.get_mut(&key).unwrap() = res;
			res
		}
		None => key,
	}
}

impl<const N: usize, const M: usize, const O: usize> From<(u64, [u64; N], [u64; M], [u64; O])>
	for Node
{
	fn from((id, f, t, o): (u64, [u64; N], [u64; M], [u64; O])) -> Self {
		Node {
			id,
			from: f.into_iter().collect(),
			to: t.into_iter().collect(),
			of: o.into_iter().collect(),
		}
	}
}

#[cfg(test)]
mod test {
	use proptest::{
		prelude::*,
		test_runner::{Config, TestRunner},
	};

	use super::*;

	impl PartialEq for Node {
		fn eq(&self, other: &Self) -> bool {
			// Deconstructed so that it will cause a
			// compilation error if we add a field and
			// forget to update this
			let Node {
				id: l_id,
				from: l_from,
				to: l_to,
				of: l_of,
			} = self;
			let Node {
				id: r_id,
				from: r_from,
				to: r_to,
				of: r_of,
			} = other;

			l_id == r_id && l_from == r_from && l_to == r_to && l_of == r_of
		}
	}

	impl Eq for Node {}

	fn compare_behaviour_compression(instructions: Vec<(u64, u64)>) -> Result<(), TestCaseError> {
		let mut fast = Graph::new();
		let mut slow = Graph::new();
		for (from, to) in instructions.into_iter() {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(&fast_.nodes, &clone.nodes);
		}

		Ok(())
	}

	fn generator(max_id: u64, instruction_count: usize) -> impl Strategy<Value = Vec<(u64, u64)>> {
		let node_pair = (0..max_id, 0..max_id);
		prop::collection::vec(node_pair, instruction_count)
	}

	#[test]
	fn compare_10_20_compression() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner
			.run(&generator(10, 20), compare_behaviour_compression)
			.unwrap();
	}

	/// 0 → 1 → 2
	#[test]
	fn straight_line() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 2 → 1 → 0
	#[test]
	fn straight_line_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [], [1], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [2, 1, 0]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0 → 1
	#[test]
	fn short_line() {
		#[rustfmt::skip]
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [], [1]).into())
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes([(0, (0, [], [], [0, 1]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 1 → 0
	#[test]
	fn short_line_rev() {
		let mut graph = Graph::with_nodes(
			[(0, (0, [1], [], [0]).into()), (1, (1, [], [0], [1]).into())]
				.into_iter()
				.collect(),
		);
		let expected = Graph::with_nodes([(0, (0, [], [], [1, 0]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   3
	#[test]
	fn diamond() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   3
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   0
	#[test]
	fn diamond_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4 → 0
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 6   3
	#[test]
	fn diamond_on_stick() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [4], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
				(4, (4, [5], [0], [4]).into()),
				(5, (5, [6], [4], [5]).into()),
				(6, (6, [], [5], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [6, 5, 4, 0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 6 → 3
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 4   0
	#[test]
	fn diamond_on_stick_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [6], [1, 2], [3]).into()),
				(4, (4, [], [5], [4]).into()),
				(5, (5, [4], [6], [5]).into()),
				(6, (6, [5], [3], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [4, 5, 6, 3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	///  ↘ ↙
	///   2
	///   ↓
	///   3
	///  ↙ ↘
	/// 4   5
	#[test]
	fn cross() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [3], [2]).into()),
				(3, (3, [2], [4, 5], [3]).into()),
				(4, (4, [3], [], [4]).into()),
				(5, (5, [3], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [4, 5], [2, 3]).into()),
				(4, (4, [2], [], [4]).into()),
				(5, (5, [2], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4   5
	///  ↘ ↙
	///   3
	///   ↓
	///   2
	///  ↙ ↘
	/// 0   1
	#[test]
	fn cross_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [3], [0, 1], [2]).into()),
				(3, (3, [4, 5], [2], [3]).into()),
				(4, (4, [], [3], [4]).into()),
				(5, (5, [], [3], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [4, 5], [0, 1], [3, 2]).into()),
				(4, (4, [], [2], [4]).into()),
				(5, (5, [], [2], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 1   3
	///  ↘ ↗
	///   2
	#[test]
	fn cycle() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [3], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [3], [2]).into()),
				(3, (3, [2], [0], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 1, 2, 3]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 3   1
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_rev() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [3], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [3], [1], [2]).into()),
				(3, (3, [0], [2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 3, 2, 1]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	/// ↓   ↓
	/// 2   3
	///  ↘ ↙
	///   4
	#[test]
	fn v() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [3], [1]).into()),
				(2, (2, [0], [4], [2]).into()),
				(3, (3, [1], [4], [3]).into()),
				(4, (4, [2, 3], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [4], [0, 2]).into()),
				(1, (1, [], [4], [1, 3]).into()),
				(4, (4, [0, 1], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress();
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0 → 1 → 2
	#[test]
	fn straight_line_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 2 → 1 → 0
	#[test]
	fn straight_line_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [], [1], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected =
			Graph::with_nodes([(0, (0, [], [], [2, 1, 0]).into())].into_iter().collect());
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   3
	#[test]
	fn diamond_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   3
	///  ↙ ↘
	/// 1   2
	///  ↘ ↙
	///   0
	#[test]
	fn diamond_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = graph.clone();
		graph.verify();
		expected.verify();
		graph.compress_with_hint(3, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4 → 0
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 6   3
	#[test]
	fn diamond_on_stick_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [4], [1, 2], [0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
				(4, (4, [5], [0], [4]).into()),
				(5, (5, [6], [4], [5]).into()),
				(6, (6, [], [5], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [1, 2], [6, 5, 4, 0]).into()),
				(1, (1, [0], [3], [1]).into()),
				(2, (2, [0], [3], [2]).into()),
				(3, (3, [1, 2], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(5, 4);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 6 → 3
	/// ↑  ↙ ↘
	/// 5 1   2
	/// ↑  ↘ ↙
	/// 4   0
	#[test]
	fn diamond_on_stick_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [6], [1, 2], [3]).into()),
				(4, (4, [], [5], [4]).into()),
				(5, (5, [4], [6], [5]).into()),
				(6, (6, [5], [3], [6]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [1, 2], [], [0]).into()),
				(1, (1, [3], [0], [1]).into()),
				(2, (2, [3], [0], [2]).into()),
				(3, (3, [], [1, 2], [4, 5, 6, 3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(5, 6);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	///  ↘ ↙
	///   2
	///   ↓
	///   3
	///  ↙ ↘
	/// 4   5
	#[test]
	fn cross_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [3], [2]).into()),
				(3, (3, [2], [4, 5], [3]).into()),
				(4, (4, [3], [], [4]).into()),
				(5, (5, [3], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [2], [1]).into()),
				(2, (2, [0, 1], [4, 5], [2, 3]).into()),
				(4, (4, [2], [], [4]).into()),
				(5, (5, [2], [], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(2, 3);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 4   5
	///  ↘ ↙
	///   3
	///   ↓
	///   2
	///  ↙ ↘
	/// 0   1
	#[test]
	fn cross_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [3], [0, 1], [2]).into()),
				(3, (3, [4, 5], [2], [3]).into()),
				(4, (4, [], [3], [4]).into()),
				(5, (5, [], [3], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [2], [], [0]).into()),
				(1, (1, [2], [], [1]).into()),
				(2, (2, [4, 5], [0, 1], [3, 2]).into()),
				(4, (4, [], [2], [4]).into()),
				(5, (5, [], [2], [5]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(3, 2);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 1   3
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [3], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [3], [2]).into()),
				(3, (3, [2], [0], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 1, 2, 3]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(0, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	///   0
	///  ↙ ↖
	/// 3   1
	///  ↘ ↗
	///   2
	#[test]
	fn cycle_rev_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [1], [3], [0]).into()),
				(1, (1, [2], [0], [1]).into()),
				(2, (2, [3], [1], [2]).into()),
				(3, (3, [0], [2], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[(0, (0, [0], [0], [0, 3, 2, 1]).into())]
				.into_iter()
				.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(2, 1);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0   1
	/// ↓   ↓
	/// 2   3
	///  ↘ ↙
	///   4
	#[test]
	fn v_hint() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [2], [0]).into()),
				(1, (1, [], [3], [1]).into()),
				(2, (2, [0], [4], [2]).into()),
				(3, (3, [1], [4], [3]).into()),
				(4, (4, [2, 3], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected = Graph::with_nodes(
			[
				(0, (0, [], [4], [0, 2]).into()),
				(1, (1, [], [3], [1]).into()),
				(3, (3, [1], [4], [3]).into()),
				(4, (4, [0, 3], [], [4]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected.verify();
		graph.compress_with_hint(2, 4);
		graph.apply_merges();
		graph.verify();
		assert_eq!(graph.nodes, expected.nodes);
	}

	/// 0 → 1 → 2
	/// ↓
	/// 3
	#[test]
	fn incremental_l() {
		let mut graph = Graph::with_nodes(
			[
				(0, (0, [], [1], [0]).into()),
				(1, (1, [0], [2], [1]).into()),
				(2, (2, [1], [], [2]).into()),
			]
			.into_iter()
			.collect(),
		);
		let expected_1 =
			Graph::with_nodes([(0, (0, [], [], [0, 1, 2]).into())].into_iter().collect());
		let expected_2 = Graph::with_nodes(
			[
				(0, (0, [], [1, 3], [0]).into()),
				(1, (1, [0], [], [1, 2]).into()),
				(3, (3, [0], [], [3]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();
		expected_1.verify();
		expected_2.verify();

		graph.compress();
		graph.apply_merges();
		assert_eq!(&graph.nodes, &expected_1.nodes);
		graph.revert_and_update(0, 3);
		graph.compress_with_hint(0, 3);
		graph.apply_merges();
		assert_eq!(&graph.nodes, &expected_2.nodes);
	}

	#[test]
	fn incremental_generated_1() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(9, 8);
		cycle(0, 9);
		cycle(1, 8);
	}

	#[test]
	fn incremental_generated_2() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(2, 0);
		cycle(2, 4);
		cycle(1, 3);
		cycle(3, 0);
		cycle(2, 3);
	}

	#[test]
	fn incremental_generated_2_rev() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(2, 0);
		cycle(2, 4);
		cycle(3, 0);
		cycle(1, 3);
		cycle(2, 3);
	}

	#[test]
	fn incremental_generated_3() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(0, 4);
		cycle(0, 2);
		cycle(8, 1);
		cycle(8, 0);
	}

	#[test]
	fn incremental_generated_4() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(0, 2);
		cycle(5, 1);
		cycle(1, 2);
		cycle(5, 2);
	}

	#[test]
	fn incremental_generated_5() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 2);
		cycle(0, 0);
		cycle(0, 5);
		cycle(2, 1);
		cycle(3, 2);
		cycle(3, 1);
	}

	#[test]
	fn incremental_generated_6() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(2, 3);
		cycle(3, 0);
		cycle(0, 4);
		cycle(1, 2);
		cycle(2, 0);
	}

	#[test]
	fn incremental_generated_7() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 8);
		cycle(8, 8);
		cycle(0, 0);
	}

	#[test]
	fn incremental_generated_8() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(0, 7);
		cycle(5, 0);
		cycle(0, 0);
		cycle(8, 3);
		cycle(0, 0);
		cycle(4, 0);
		cycle(3, 7);
		cycle(0, 2);
		cycle(7, 1);
		cycle(0, 3);
		cycle(0, 0);
	}

	#[test]
	fn incremental_generated_9() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 0);
		cycle(8, 2);
		cycle(2, 4);
		cycle(4, 2);
		cycle(0, 4);
	}

	#[test]
	fn incremental_generated_10() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(9, 6);
		cycle(0, 3);
		cycle(2, 9);
		cycle(3, 3);
		cycle(6, 2);
	}

	#[test]
	fn incremental_generated_11() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(1, 0);
		cycle(2, 2);
		cycle(6, 1);
		cycle(0, 1);
		cycle(0, 7);
	}

	#[test]
	fn incremental_generated_12() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(0, 5);
		cycle(1, 0);
		cycle(5, 1);
		cycle(5, 5);
		cycle(0, 0);
	}

	#[test]
	fn incremental_generated_13() {
		let mut slow = Graph::new();
		let mut fast = Graph::new();

		let mut cycle = |from, to| {
			slow.update(from, to);
			fast.revert_and_update(from, to);
			fast.compress_with_hint(from, to);

			let mut fast_ = fast.clone();
			fast_.apply_merges();

			let mut clone = slow.clone();
			clone.compress();
			clone.apply_merges();

			assert_eq!(fast_.nodes, clone.nodes);
		};

		cycle(4, 8);
		cycle(7, 4);
		cycle(8, 2);
		cycle(9, 7);
		cycle(1, 9);
		cycle(2, 2);
		cycle(2, 0);
		cycle(0, 0);
		slow.update(9, 7);
		fast.revert_and_update(9, 7);
		fast.compress_with_hint(9, 7);

		let mut fast_ = fast.clone();
		fast_.apply_merges();

		let mut clone = slow.clone();
		clone.compress();
		clone.apply_merges();

		assert_eq!(fast_.nodes, clone.nodes);

		//cycle(9, 7);
	}

	// (1, 0), (2, 2), (6, 1), (0, 1), (0, 0)
	// (0, 5), (1, 0), (5, 1), (1, 0), (5, 5), (0, 0),

	// (4, 8), (7, 4), (8, 2), (9, 7), (1, 9), (2, 2), (2, 0), (2, 0), (2, 0), (0, 0), (9, 7),

	// (3, 2), (2, 7), (0, 3), (7, 9), (9, 1), (3, 3), (0, 2), (7, 9),

	/// [Image](https://upload.wikimedia.org/wikipedia/commons/e/e1/Scc-1.svg)
	#[test]
	fn strongly_connected_graph_small_tarjan() {
		let graph = Graph::with_nodes(
			[
				(0, (0, [4], [1], [0]).into()),
				(1, (1, [0], [2, 4, 5], [1]).into()),
				(2, (2, [1, 3], [3, 6], [2]).into()),
				(3, (3, [2, 7], [2, 7], [3]).into()),
				(4, (4, [1], [0, 5], [4]).into()),
				(5, (5, [1, 4, 6], [6], [5]).into()),
				(6, (6, [2, 5, 7], [5], [6]).into()),
				(7, (7, [3], [3, 6], [7]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();

		let expected = Graph::with_nodes(
			[
				(0, (0, [], [2, 5], [0, 1, 4]).into()),
				(2, (2, [0], [5], [2, 3, 7]).into()),
				(5, (5, [0, 2], [], [5, 6]).into()),
			]
			.into_iter()
			.collect(),
		);
		expected.verify();

		let result = graph.to_strongly_connected_components_tarjan();
		assert_eq!(result.nodes, expected.nodes);
	}

	/// [Image](https://upload.wikimedia.org/wikipedia/commons/2/20/Graph_Condensation.svg)
	#[test]
	fn strongly_connected_graph_large_tarjan() {
		let graph = Graph::with_nodes(
			[
				(0, (0, [1], [2], [0]).into()),
				(1, (1, [2], [0, 5], [1]).into()),
				(2, (2, [0, 3], [1, 4], [2]).into()),
				(3, (3, [4], [2, 9], [3]).into()),
				(4, (4, [2], [3, 5, 10], [4]).into()),
				(5, (5, [1, 4], [6, 8, 13], [5]).into()),
				(6, (6, [5, 8], [7], [6]).into()),
				(7, (7, [6], [8, 15], [7]).into()),
				(8, (8, [5, 7], [6, 15], [8]).into()),
				(9, (9, [3, 11], [10], [9]).into()),
				(10, (10, [9, 4], [11, 12], [10]).into()),
				(11, (11, [10, 12], [9], [11]).into()),
				(12, (12, [10], [11, 13], [12]).into()),
				(13, (13, [5, 12, 14], [14, 15], [13]).into()),
				(14, (14, [13], [13], [14]).into()),
				(15, (15, [7, 8, 13], [], [15]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();

		let expected = Graph::with_nodes(
			[
				(0, (0, [], [5, 9], [0, 1, 2, 3, 4]).into()),
				(5, (5, [0], [6, 13], [5]).into()),
				(6, (6, [5], [15], [6, 7, 8]).into()),
				(9, (9, [0], [13], [9, 10, 11, 12]).into()),
				(13, (13, [5, 9], [15], [13, 14]).into()),
				(15, (15, [6, 13], [], [15]).into()),
			]
			.into_iter()
			.collect(),
		);
		expected.verify();

		let result = graph.to_strongly_connected_components_tarjan();
		assert_eq!(result.nodes, expected.nodes);
	}

	/// [Image](https://upload.wikimedia.org/wikipedia/commons/e/e1/Scc-1.svg)
	#[test]
	fn strongly_connected_graph_small_kosaraju() {
		let graph = Graph::with_nodes(
			[
				(0, (0, [4], [1], [0]).into()),
				(1, (1, [0], [2, 4, 5], [1]).into()),
				(2, (2, [1, 3], [3, 6], [2]).into()),
				(3, (3, [2, 7], [2, 7], [3]).into()),
				(4, (4, [1], [0, 5], [4]).into()),
				(5, (5, [1, 4, 6], [6], [5]).into()),
				(6, (6, [2, 5, 7], [5], [6]).into()),
				(7, (7, [3], [3, 6], [7]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();

		let expected = Graph::with_nodes(
			[
				(0, (0, [], [2, 5], [0, 1, 4]).into()),
				(2, (2, [0], [5], [2, 3, 7]).into()),
				(5, (5, [0, 2], [], [5, 6]).into()),
			]
			.into_iter()
			.collect(),
		);
		expected.verify();

		let result = graph.to_strongly_connected_components_kosaraju();
		assert_eq!(result.nodes, expected.nodes);
	}

	/// [Image](https://upload.wikimedia.org/wikipedia/commons/2/20/Graph_Condensation.svg)
	#[test]
	fn strongly_connected_graph_large_kosaraju() {
		let graph = Graph::with_nodes(
			[
				(0, (0, [1], [2], [0]).into()),
				(1, (1, [2], [0, 5], [1]).into()),
				(2, (2, [0, 3], [1, 4], [2]).into()),
				(3, (3, [4], [2, 9], [3]).into()),
				(4, (4, [2], [3, 5, 10], [4]).into()),
				(5, (5, [1, 4], [6, 8, 13], [5]).into()),
				(6, (6, [5, 8], [7], [6]).into()),
				(7, (7, [6], [8, 15], [7]).into()),
				(8, (8, [5, 7], [6, 15], [8]).into()),
				(9, (9, [3, 11], [10], [9]).into()),
				(10, (10, [9, 4], [11, 12], [10]).into()),
				(11, (11, [10, 12], [9], [11]).into()),
				(12, (12, [10], [11, 13], [12]).into()),
				(13, (13, [5, 12, 14], [14, 15], [13]).into()),
				(14, (14, [13], [13], [14]).into()),
				(15, (15, [7, 8, 13], [], [15]).into()),
			]
			.into_iter()
			.collect(),
		);
		graph.verify();

		let expected = Graph::with_nodes(
			[
				(0, (0, [], [5, 9], [0, 1, 2, 3, 4]).into()),
				(5, (5, [0], [6, 13], [5]).into()),
				(6, (6, [5], [15], [6, 7, 8]).into()),
				(9, (9, [0], [13], [9, 10, 11, 12]).into()),
				(13, (13, [5, 9], [15], [13, 14]).into()),
				(15, (15, [6, 13], [], [15]).into()),
			]
			.into_iter()
			.collect(),
		);
		expected.verify();

		let result = graph.to_strongly_connected_components_kosaraju();
		assert_eq!(result.nodes, expected.nodes);
	}

	#[test]
	fn tarjan_kosaraju_eq_small() {
		let graph = Graph::with_nodes(
			[
				(0, (0, [4], [1], [0]).into()),
				(1, (1, [0], [2, 4, 5], [1]).into()),
				(2, (2, [1, 3], [3, 6], [2]).into()),
				(3, (3, [2, 7], [2, 7], [3]).into()),
				(4, (4, [1], [0, 5], [4]).into()),
				(5, (5, [1, 4, 6], [6], [5]).into()),
				(6, (6, [2, 5, 7], [5], [6]).into()),
				(7, (7, [3], [3, 6], [7]).into()),
			]
			.into_iter()
			.collect(),
		);

		let t = graph.tarjan();
		let k = graph.kosaraju();

		assert_eq!(t, k);
	}

	fn compare_behaviour_scc(instructions: Vec<(u64, u64)>) -> Result<(), TestCaseError> {
		let mut graph = Graph::new();
		for (from, to) in instructions.into_iter() {
			graph.update(from, to);
		}

		let t = graph.to_strongly_connected_components_tarjan().nodes;
		let k = graph.to_strongly_connected_components_kosaraju().nodes;

		assert_eq!(t, k);

		Ok(())
	}

	#[test]
	fn compare_10_20_scc() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner
			.run(&generator(10, 20), compare_behaviour_scc)
			.unwrap();
	}

	#[test]
	fn compare_100_2000_scc() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner
			.run(&generator(100, 2000), compare_behaviour_scc)
			.unwrap();
	}

	#[test]
	fn compare_75k_100k_scc() {
		let nodes = 75_000;
		let edges = 100_000;

		let mut graph = Graph::new();
		for (from, to) in
			std::iter::from_fn(|| Some((fastrand::u64(..nodes), fastrand::u64(..nodes))))
				.take(edges)
		{
			graph.update(from, to);
		}

		let t = graph.to_strongly_connected_components_tarjan().nodes;
		let k = graph.to_strongly_connected_components_kosaraju().nodes;

		assert_eq!(t, k);
	}
}
