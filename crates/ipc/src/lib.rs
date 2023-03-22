use std::{
	io::{self, BufReader, BufWriter, Read, Write},
	mem,
};

use serde::{Deserialize, Serialize};

pub struct Ipc<S: Copy + Read + Write> {
	stream_tx: BufWriter<S>,
	stream_rx: BufReader<S>,
}

impl<S: Copy + Read + Write> Ipc<S> {
	pub fn new(stream: S) -> Self {
		Self {
			stream_tx: BufWriter::new(stream),
			stream_rx: BufReader::new(stream),
		}
	}

	pub fn blocking_send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
		let size = bincode::serialized_size(msg).unwrap();
		self.stream_tx.write_all(&size.to_le_bytes())?;
		bincode::serialize_into(&mut self.stream_tx, msg)?;
		self.stream_tx.flush()?;
		Ok(())
	}

	pub fn blocking_receive(&mut self) -> Result<IpcMessage, IpcError> {
		let size = {
			let mut size = [0u8; mem::size_of::<u64>()];
			self.stream_rx.read_exact(&mut size)?;
			u64::from_le_bytes(size)
		};
		let ret = bincode::deserialize_from((&mut self.stream_rx).take(size))?;
		Ok(ret)
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IpcMessage {
	Ping,
}

#[derive(Debug)]
pub enum IpcError {
	EndOfFile,
	Interrupted,
	Io(io::Error),
}

impl From<io::Error> for IpcError {
	fn from(err: io::Error) -> Self {
		match err.kind() {
			io::ErrorKind::Interrupted => Self::Interrupted,
			io::ErrorKind::UnexpectedEof => Self::EndOfFile,
			_ => Self::Io(err),
		}
	}
}
impl From<bincode::Error> for IpcError {
	fn from(err: bincode::Error) -> Self {
		match *err {
			bincode::ErrorKind::Io(io) => Self::from(io),
			other => panic!("bincode error `{:?}`", other),
		}
	}
}
