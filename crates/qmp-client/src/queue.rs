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
		assert!(self.end > skip_at_end);
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
