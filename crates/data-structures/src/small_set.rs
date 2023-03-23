use std::{collections::BTreeSet, iter, mem};

use arrayvec::ArrayVec;

const ACTUAL_SIZE: usize = 6;

/// A small size optimised u64 set
#[derive(Debug, Clone)]
pub enum SmallU64Set {
	Set(BTreeSet<u64>),
	Vec(ArrayVec<u64, ACTUAL_SIZE>),
}

impl Default for SmallU64Set {
	fn default() -> Self {
		Self::new()
	}
}

impl IntoIterator for SmallU64Set {
	type IntoIter = itertools::Either<
		<BTreeSet<u64> as IntoIterator>::IntoIter,
		<ArrayVec<u64, ACTUAL_SIZE> as IntoIterator>::IntoIter,
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

impl From<SmallU64Set> for BTreeSet<u64> {
	fn from(val: SmallU64Set) -> Self {
		match val {
			SmallU64Set::Set(s) => s,
			SmallU64Set::Vec(v) => v.into_iter().collect(),
		}
	}
}

#[cfg(test)]
impl PartialEq for SmallU64Set {
	fn eq(&self, other: &Self) -> bool {
		let l: BTreeSet<_> = self.clone().into();
		let r: BTreeSet<_> = other.clone().into();
		l == r
	}
}

#[cfg(test)]
impl Eq for SmallU64Set {}

impl SmallU64Set {
	pub fn new() -> Self {
		SmallU64Set::Vec(ArrayVec::new())
	}

	// Returns an arbitrary element in the set. Panics if the set is empty
	pub fn get_any(&self) -> u64 {
		match self {
			SmallU64Set::Set(s) => *s.iter().next().unwrap(),
			SmallU64Set::Vec(v) => v[0],
		}
	}

	pub fn insert(&mut self, val: u64) -> bool {
		match self {
			SmallU64Set::Set(s) => s.insert(val),
			SmallU64Set::Vec(v) => {
				let contains = vec_contains(v, val);
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
						v.remove(idx); // Must upload sorting
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

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn contains(&self, val: &u64) -> bool {
		match self {
			SmallU64Set::Set(s) => s.contains(val),
			SmallU64Set::Vec(v) => vec_contains(v, *val),
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &u64> {
		match self {
			SmallU64Set::Set(s) => itertools::Either::Left(s.iter()),
			SmallU64Set::Vec(v) => itertools::Either::Right(v.iter()),
		}
	}
}

fn vec_contains(slice: &[u64], val: u64) -> bool {
	slice.iter().any(|&x| x == val)
}
