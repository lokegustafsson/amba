use std::{
	borrow::Cow,
	io::{self, BufReader, BufWriter, Read, Write},
	mem,
	net::Shutdown,
	os::unix::net::UnixStream,
};

use serde::{Deserialize, Serialize};

pub use crate::graph::{GraphIpc, GraphIpcBuilder, NodeMetadata};

mod graph;

pub fn new_wrapping(stream: &UnixStream) -> (IpcTx<'_>, IpcRx<'_>) {
	(
		IpcTx {
			tx: BufWriter::new(stream),
		},
		IpcRx {
			rx: BufReader::new(stream),
		},
	)
}

pub struct IpcTx<'a> {
	tx: BufWriter<&'a UnixStream>,
}

impl Drop for IpcTx<'_> {
	fn drop(&mut self) {
		match self.tx.get_ref().shutdown(Shutdown::Write) {
			Ok(()) => {}
			Err(error) => tracing::error!(?error, "failed shutting down IpcTx on drop"),
		}
	}
}

impl IpcTx<'_> {
	pub fn blocking_send(&mut self, msg: &IpcMessage<'_>) -> Result<(), IpcError> {
		let size = bincode::serialized_size(msg).unwrap();
		self.tx.write_all(&size.to_le_bytes())?;
		bincode::serialize_into(&mut self.tx, msg)?;
		self.tx.flush()?;
		Ok(())
	}
}

pub struct IpcRx<'a> {
	rx: BufReader<&'a UnixStream>,
}

impl Drop for IpcRx<'_> {
	fn drop(&mut self) {
		match self.rx.get_ref().shutdown(Shutdown::Read) {
			Ok(()) => {}
			Err(error) => tracing::error!(?error, "failed shutting down IpcRx on drop"),
		}
	}
}

impl IpcRx<'_> {
	pub fn blocking_receive(&mut self) -> Result<IpcMessage<'static>, IpcError> {
		let size = {
			let mut size = [0u8; mem::size_of::<u64>()];
			self.rx.read_exact(&mut size)?;
			u64::from_le_bytes(size)
		};
		let ret = bincode::deserialize_from((&mut self.rx).take(size))?;
		Ok(ret)
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IpcMessage<'a> {
	Ping,
	GraphSnapshot {
		symbolic_state_graph: Cow<'a, GraphIpc>,
		basic_block_graph: Cow<'a, GraphIpc>,
	},
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
