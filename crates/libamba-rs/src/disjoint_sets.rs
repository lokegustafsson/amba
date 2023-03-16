use std::{collections::HashMap, mem};

/// A disjoint-sets data structure
///
/// See <https://en.wikipedia.org/wiki/Disjoint-set_data_structure>
///
/// Conceptually we start with an empty graph with 2^63 nodes numbered 0, 1, ..
/// We have two operations that take amortized near-constant time:
/// - Merge: Connecting nodes `a` and `b` with an edge
/// - Canonicalize: Querying the minimum node within the component including `x`
#[derive(Default)]
pub struct DisjointSets {
	parent_or_size: HashMap<u64, u64>,
	root_names: HashMap<u64, u64>,
}
const LIMIT: u64 = 1 << 63;

impl DisjointSets {
	/// Merge the groups containing elements `a` and `b`
	pub fn merge(&mut self, mut a: u64, mut b: u64) -> bool {
		assert!(a < LIMIT);
		assert!(b < LIMIT);

		a = self.find(a);
		b = self.find(b);
		if a == b {
			return false;
		}
		let size_a = self.parent_or_size[&a] - LIMIT;
		let size_b = self.parent_or_size[&b] - LIMIT;
		if size_a < size_b {
			mem::swap(&mut a, &mut b);
		}
		self.parent_or_size.insert(a, LIMIT + size_a + size_b);
		self.parent_or_size.insert(b, a);
		{
			let name_b = self.root_names.get(&b).copied().unwrap_or(b);
			let name_a = self.root_names.entry(a).or_insert(a);
			min_eq(name_a, name_b);
		}
		true
	}

	/// Retrieve the root element of the group containing `x`
	fn find(&mut self, mut x: u64) -> u64 {
		assert!(x < LIMIT);

		let mut along_the_way = {
			let y = *self.parent_or_size.entry(x).or_insert(LIMIT + 1);
			if y >= LIMIT {
				return x;
			}
			let original_x = x;
			x = y;
			vec![original_x]
		};

		loop {
			let y = self.parent_or_size[&x];
			if y < LIMIT {
				along_the_way.push(x);
			} else {
				along_the_way.iter().for_each(|&prev| {
					self.parent_or_size.insert(prev, x);
				});
				return x;
			}
		}
	}

	/// Retrieve the minimum element of the group containing `x`
	pub fn canonicalize(&mut self, x: u64) -> u64 {
		let x = self.find(x);
		self.root_names.get(&x).copied().unwrap_or(x)
	}

	/// Do `a` and `b` belong to the same group?
	pub fn same_set(&mut self, a: u64, b: u64) -> bool {
		self.canonicalize(a) == self.canonicalize(b)
	}
}

fn min_eq(slot: &mut u64, v: u64) {
	*slot = (*slot).min(v);
}

#[cfg(test)]
mod test {
	use std::collections::HashSet;

	use proptest::{
		prelude::*,
		test_runner::{Config, TestRunner},
	};

	use super::*;

	#[derive(Default)]
	struct Slow {
		/// Map element to representative
		canon: HashMap<u64, u64>,
		/// Map representative to elements
		sets: HashMap<u64, HashSet<u64>>,
	}
	impl Slow {
		pub fn merge(&mut self, mut a: u64, mut b: u64) -> bool {
			a = self.canonicalize(a);
			b = self.canonicalize(b);
			if a == b {
				return false;
			}
			if a > b {
				mem::swap(&mut a, &mut b);
			}
			for bb in self.sets.remove(&b).unwrap() {
				self.canon.insert(bb, a);
				self.sets.get_mut(&a).unwrap().insert(bb);
			}
			true
		}

		pub fn canonicalize(&mut self, mut x: u64) -> u64 {
			assert!(x < LIMIT);
			if !self.canon.contains_key(&x) {
				self.canon.insert(x, x);
				self.sets.insert(x, [x].into_iter().collect());
			}
			self.canon[&x]
		}
	}

	#[derive(Debug)]
	enum Instruction {
		Merge(u64, u64),
		Canonicalize(u64),
	}
	fn compare_behavior(instructions: Vec<Instruction>) -> Result<(), TestCaseError> {
		let mut slow = Slow::default();
		let mut fast = DisjointSets::default();
		assert!(LIMIT.is_power_of_two());
		for ins in instructions {
			match ins {
				Instruction::Merge(a, b) => {
					let r1 = slow.merge(a, b);
					let r2 = fast.merge(a, b);
					assert_eq!(r1, r2, "different merge behavior");
				}
				Instruction::Canonicalize(x) => {
					let r1 = slow.canonicalize(x);
					let r2 = fast.canonicalize(x);
					assert_eq!(r1, r2, "different canonicalize behavior");
				}
			}
		}
		Ok(())
	}

	fn generator(elements: u64, instructions: usize) -> impl Strategy<Value = Vec<Instruction>> {
		assert!(elements <= LIMIT);
		let instruction = prop_oneof![
			(0..elements, 0..elements).prop_map(|(a, b)| Instruction::Merge(a, b)),
			(0..elements).prop_map(|x| Instruction::Canonicalize(x)),
		];
		prop::collection::vec(instruction, 0..instructions)
	}

	#[test]
	fn compare_1_20() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(1, 20), compare_behavior).unwrap();
	}
	#[test]
	fn compare_2_20() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(2, 20), compare_behavior).unwrap();
	}
	#[test]
	fn compare_3_20() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(3, 20), compare_behavior).unwrap();
	}
	#[test]
	fn compare_4_4() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(4, 4), compare_behavior).unwrap();
	}
	#[test]
	fn compare_5_3() {
		let mut runner = TestRunner::new(Config::with_cases(20_000));
		runner.run(&generator(5, 3), compare_behavior).unwrap();
	}
	#[test]
	fn compare_10_3() {
		let mut runner = TestRunner::new(Config::with_cases(20_000));
		runner.run(&generator(10, 3), compare_behavior).unwrap();
	}
	#[test]
	fn compare_all_3() {
		let mut runner = TestRunner::new(Config::with_cases(20_000));
		runner.run(&generator(LIMIT, 3), compare_behavior).unwrap();
	}
}
