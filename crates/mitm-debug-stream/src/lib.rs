use std::io::{self, Read, Write};

#[derive(Clone)]
pub struct Mitm<S: Read + Write>(pub S);

impl<S: Read + Write> Read for Mitm<S> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		tracing::debug!(buf_len = buf.len(), "attempt read");
		let len = self.0.read(buf);
		let printbuf = match len {
			Ok(l) => String::from_utf8_lossy(&buf[..l]),
			Err(_) => String::from_utf8_lossy(&[]),
		};
		tracing::debug!(?len, buf = &*printbuf, "read");
		len
	}
}
impl<S: Read + Write> Write for Mitm<S> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		tracing::debug!(buf_len = buf.len(), "attempt write");
		let len = self.0.write(buf);
		tracing::debug!(
			?len,
			buf = &*String::from_utf8_lossy(buf),
			"write"
		);
		len
	}

	fn flush(&mut self) -> io::Result<()> {
		tracing::debug!("attempt flush");
		let len = self.0.flush();
		tracing::debug!(?len, "flush");
		len
	}
}
