use std::io::{BufRead, Read, Result};

pub struct ReadUntil<R> {
	inner: R,
	end: u8,
	eof: bool,
}
impl<R: BufRead> Read for ReadUntil<R> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		if self.eof {
			return Ok(0);
		}
		let src = self.inner.fill_buf()?;
		let limit = src
			.iter()
			.enumerate()
			.find_map(|(i, &b)| (b == self.end).then_some(i + 1));
		let len = limit.unwrap_or(src.len()).min(buf.len());
		buf[..len].copy_from_slice(&src[..len]);
		self.inner.consume(len);
		if limit == Some(len) {
			self.eof = true;
		}
		Ok(len)
	}
}

pub trait ReadExt: Sized {
	fn take_until(&mut self, end: u8) -> ReadUntil<&mut Self>;
}
impl<R: BufRead> ReadExt for R {
	fn take_until(&mut self, end: u8) -> ReadUntil<&mut Self> {
		ReadUntil {
			inner: self,
			end,
			eof: false,
		}
	}
}
