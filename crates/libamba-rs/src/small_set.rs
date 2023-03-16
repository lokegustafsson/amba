use std::{collections::BTreeSet, iter, mem};

use smallvec::SmallVec;

const OPTIMAL_SIZE: usize = {
	let set_size = mem::size_of::<BTreeSet<u64>>();
	let u64_size = mem::size_of::<u64>();
	set_size / u64_size
};
const ACTUAL_SIZE: usize = 5;

/// A small size optimised u64 set
// Hardcoding one two and three let's us fit three elements without
// growing larger than a BTreeSet and use the enum discriminant as an index
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmallU64Set {
	Set(BTreeSet<u64>),
	Vec(SmallVec<[u64; ACTUAL_SIZE]>),
}

impl SmallU64Set {
	pub fn new() -> Self {
		SmallU64Set::Vec(SmallVec::new())
	}

	pub fn insert(&mut self, val: u64) -> bool {
		match self {
			SmallU64Set::Set(s) => s.insert(val),
			SmallU64Set::Vec(v) => {
				let contains = vec_contains(&v, val);
				if contains {
					return false;
				}

				if v.len() == ACTUAL_SIZE {
					let s = v.iter().copied().chain(iter::once(val)).collect();
					*self = SmallU64Set::Set(s);
				} else {
					v.push(val);
				}

				true
			}
		}
	}

	pub fn remove(&mut self, val: &u64) -> bool {
		match self {
			SmallU64Set::Set(s) => s.remove(val),
			SmallU64Set::Vec(v) => {
				let val = *val;
				match v.iter().position(|&x| x == val) {
					Some(idx) => {
						v.swap_remove(idx);
						true
					}
					None => false,
				}
			}
		}
	}

	pub fn len(&self) -> usize {
		match self {
			SmallU64Set::Set(s) => s.len(),
			SmallU64Set::Vec(v) => v.len(),
		}
	}

	pub fn contains(&self, val: &u64) -> bool {
		match self {
			SmallU64Set::Set(s) => s.contains(val),
			SmallU64Set::Vec(v) => vec_contains(&v, *val),
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &u64> {
		match self {
			SmallU64Set::Set(s) => itertools::Either::Left(s.iter()),
			SmallU64Set::Vec(v) => itertools::Either::Right(v.iter()),
		}
	}
}

impl Default for SmallU64Set {
	fn default() -> Self {
		Self::new()
	}
}

impl IntoIterator for SmallU64Set {
	type IntoIter = itertools::Either<
		<BTreeSet<u64> as IntoIterator>::IntoIter,
		<SmallVec<[u64; ACTUAL_SIZE]> as IntoIterator>::IntoIter,
	>;
	type Item = u64;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			SmallU64Set::Set(s) => itertools::Either::Left(s.into_iter()),
			SmallU64Set::Vec(v) => itertools::Either::Right(v.into_iter()),
		}
	}
}

impl FromIterator<u64> for SmallU64Set {
	fn from_iter<T: IntoIterator<Item = u64>>(iter: T) -> Self {
		let mut s = SmallU64Set::new();
		for i in iter.into_iter() {
			s.insert(i);
		}
		s
	}
}

fn vec_contains(slice: &[u64], val: u64) -> bool {
	slice.iter().any(|&x| x == val)
}

#[cfg(test)]
mod test {
	use crate::small_set::*;

	#[test]
	fn print_optimal_size() {
		println!("{OPTIMAL_SIZE}");
		panic!()
	}
}
