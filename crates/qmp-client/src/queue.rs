/// Dynamically growing, non-atomic circular buffer.
pub struct ByteQueue {
	buf: Vec<u8>,
	mask: usize,
	start: usize,
	end: usize,
}

impl ByteQueue {
	pub fn with_capacity(capacity: usize) -> Self {
		let capacity = capacity.next_power_of_two();
		assert!(capacity > 0);
		Self {
			buf: vec![0u8; capacity],
			mask: capacity - 1,
			start: 0,
			end: 0,
		}
	}

	pub fn slice_to_write(&mut self) -> &mut [u8] {
		if self.end - self.start == self.mask + 1 {
			let capacity = self.mask + 1;
			let new_capacity = capacity.checked_mul(2).unwrap();
			let mut new_buf = vec![0u8; new_capacity];

			let mid = self.start & self.mask;
			let len_end = capacity - mid;
			new_buf[..len_end].copy_from_slice(&self.buf[mid..]);
			new_buf[len_end..capacity].copy_from_slice(&self.buf[..mid]);
			*self = Self {
				buf: new_buf,
				start: 0,
				end: capacity,
				mask: new_capacity - 1,
			};
			return &mut self.buf[capacity..];
		}
		let start_idx = self.start & self.mask;
		let end_idx = self.end & self.mask;
		if start_idx > end_idx {
			&mut self.buf[end_idx..start_idx]
		} else {
			&mut self.buf[end_idx..]
		}
	}

	pub fn commit_written(&mut self, written: usize) {
		self.end += written;
		assert!(self.end - self.start <= self.buf.len());
	}

	pub fn consume_slices_skipping_end_bytes(&mut self, skip_at_end: usize) -> (&[u8], &[u8]) {
		assert!(self.end >= skip_at_end);
		if self.start >= self.end - skip_at_end {
			return (&[], &[]);
		}
		let start_idx = self.start & self.mask;
		let end_idx = (self.end - skip_at_end) & self.mask;

		if skip_at_end == 0 {
			self.start = 0;
			self.end = 0;
		} else {
			self.start = self.end - skip_at_end;
		}
		if start_idx < end_idx {
			(&self.buf[start_idx..end_idx], &[])
		} else {
			(&self.buf[start_idx..], &self.buf[..end_idx])
		}
	}
}

#[cfg(test)]
mod test {
	use std::{collections::VecDeque, mem};

	use proptest::{
		prelude::*,
		test_runner::{Config, TestRunner},
	};

	use super::*;

	fn queue_write_all(queue: &mut ByteQueue, mut data: &[u8]) {
		loop {
			let target = queue.slice_to_write();
			if data.len() <= target.len() {
				target[..data.len()].copy_from_slice(data);
				mem::drop(target);
				queue.commit_written(data.len());
				return;
			} else {
				let (a, b) = data.split_at(target.len());
				target.copy_from_slice(a);
				data = b;
				let written = target.len();
				mem::drop(target);
				queue.commit_written(written);
			}
		}
	}

	#[derive(Debug)]
	enum Instruction {
		Read { skip: usize },
		Write { data: Vec<u8> },
	}

	fn compare_behavior(instructions: Vec<Instruction>) -> Result<(), TestCaseError> {
		let mut slow = VecDeque::<u8>::new();
		let mut fast = ByteQueue::with_capacity(16);
		for val in instructions {
			match val {
				Instruction::Write { data } => {
					slow.extend(&*data);
					queue_write_all(&mut fast, &data);
				}
				Instruction::Read { skip } => {
					let skip = skip.min(slow.len());
					let a: Vec<u8> = slow.drain(0..(slow.len() - skip)).collect();
					let b: Vec<u8> = {
						let (x, y) = fast.consume_slices_skipping_end_bytes(skip);
						Iterator::chain(x.iter(), y)
					}
					.copied()
					.collect();
					assert_eq!(a, b);
				}
			}
		}
		Ok(())
	}

	fn generator(size: usize, count: usize) -> impl Strategy<Value = Vec<Instruction>> {
		let instruction = prop_oneof![
			(0..=size).prop_map(|skip| Instruction::Read { skip }),
			prop::collection::vec(0..=255u8, 0..=size).prop_map(|data| Instruction::Write { data }),
		];
		prop::collection::vec(instruction, 0..=count)
	}

	#[test]
	fn compare_0_100() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(0, 100), compare_behavior).unwrap();
	}
	#[test]
	fn compare_1_100() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(1, 100), compare_behavior).unwrap();
	}
	#[test]
	fn compare_2_100() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(2, 100), compare_behavior).unwrap();
	}
	#[test]
	fn compare_5_100() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(5, 100), compare_behavior).unwrap();
	}
	#[test]
	fn compare_10_100() {
		let mut runner = TestRunner::new(Config::with_cases(10_000));
		runner.run(&generator(10, 100), compare_behavior).unwrap();
	}
	#[test]
	fn compare_1000_100() {
		let mut runner = TestRunner::new(Config::with_cases(1_000));
		runner.run(&generator(1000, 100), compare_behavior).unwrap();
	}
}
