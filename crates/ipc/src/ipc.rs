use std::{
	io::{self, BufRead, BufReader, BufWriter, Read, Write},
	mem,
	net::Shutdown,
	os::unix::net::{UnixListener, UnixStream},
	path::Path,
	time::Duration,
};

use io_arc::IoArc;
use serde::{Deserialize, Serialize};

pub use crate::{graph::GraphIpc, metadata::NodeMetadata};

pub struct IpcInstance {
	reader: IpcRx,
	writer: IpcTx,
}

impl IpcInstance {
	pub fn new_plugin(socket: &Path) -> Self {
		let stream = IoArc::new(UnixStream::connect(socket).unwrap());

		let reader = {
			let rx = BufReader::new(stream.clone());
			rx.get_ref()
				.as_ref()
				.set_read_timeout(Some(Duration::from_nanos(1)))
				.unwrap();
			IpcRx { rx }
		};

		let writer = IpcTx {
			tx: BufWriter::new(stream),
		};

		println!("Plugin IPC setup");
		IpcInstance { reader, writer }
	}

	pub fn new_gui(socket: &Path) -> Self {
		let ipc_listener = UnixListener::bind(socket).unwrap();
		let stream = IoArc::new(ipc_listener.accept().unwrap().0);

		let reader = {
			let rx = BufReader::new(stream.clone());
			rx.get_ref()
				.as_ref()
				.set_read_timeout(Some(Duration::from_nanos(1)))
				.unwrap();
			IpcRx { rx }
		};

		let writer = IpcTx {
			tx: BufWriter::new(stream),
		};

		println!("GUI IPC setup");
		IpcInstance { reader, writer }
	}

	pub fn get_rx_tx(&mut self) -> (&mut IpcRx, &mut IpcTx) {
		(&mut self.reader, &mut self.writer)
	}
}

pub struct IpcTx {
	tx: BufWriter<IoArc<UnixStream>>,
}

impl Drop for IpcTx {
	fn drop(&mut self) {
		match self.tx.get_ref().as_ref().shutdown(Shutdown::Write) {
			Ok(()) => {}
			Err(error) => tracing::error!(?error, "failed shutting down IpcTx on drop"),
		}
	}
}

impl IpcTx {
	pub fn blocking_send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
		let size = bincode::serialized_size(msg).unwrap();
		self.tx.write_all(&size.to_le_bytes())?;
		bincode::serialize_into(&mut self.tx, msg)?;
		self.tx.flush()?;
		Ok(())
	}
}

pub struct IpcRx {
	rx: BufReader<IoArc<UnixStream>>,
}

impl Drop for IpcRx {
	fn drop(&mut self) {
		match self.rx.get_ref().as_ref().shutdown(Shutdown::Read) {
			Ok(()) => {}
			Err(error) => tracing::error!(?error, "failed shutting down IpcRx on drop"),
		}
	}
}

impl IpcRx {
	pub fn blocking_receive(&mut self) -> Result<IpcMessage, IpcError> {
		self.rx.get_ref().as_ref().set_read_timeout(None).unwrap();
		let ret = (|| {
			let size = {
				let mut size = [0u8; mem::size_of::<u64>()];
				self.rx.read_exact(&mut size)?;
				u64::from_le_bytes(size)
			};
			bincode::deserialize_from((&mut self.rx).take(size)).map_err(Into::into)
		})();
		self.rx
			.get_ref()
			.as_ref()
			.set_read_timeout(Some(Duration::from_nanos(1)))
			.unwrap();
		ret
	}

	/// Breaks when receiving the following, recover by calling `blocking_receive`:
	/// * An incomplete packet (`IpcError::PollingReceiveFragmented`)
	/// * An packet larger than the buffer size (`IpcError::PollingReceiveTooLarge`)
	pub fn polling_receive(&mut self) -> Result<Option<IpcMessage>, IpcError> {
		let buf_capacity = self.rx.capacity();

		match self.rx.fill_buf() {
			Err(err)
				if matches!(
					err.kind(),
					io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
				) =>
			{
				Ok(None)
			}
			Err(err) => Err(IpcError::from(err)),
			Ok(&[]) => Err(IpcError::EndOfFile),
			Ok(view) => {
				if view.len() < mem::size_of::<u64>() {
					return Err(IpcError::PollingReceiveFragmented);
				}
				let header_size = mem::size_of::<u64>();
				let size = u64::from_le_bytes(view[..header_size].try_into().unwrap());
				let packet_size =
					usize::try_from(size + u64::try_from(header_size).unwrap()).unwrap();
				if packet_size > buf_capacity {
					return Err(IpcError::PollingReceiveTooLarge);
				}
				if view.len() < packet_size {
					return Err(IpcError::PollingReceiveFragmented);
				}
				let ret = bincode::deserialize(&view[header_size..packet_size])?;
				self.rx.consume(packet_size);
				Ok(Some(ret))
			}
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IpcMessage {
	Ping,
	NewEdges {
		state_edges: Vec<(NodeMetadata, NodeMetadata)>,
		block_edges: Vec<(NodeMetadata, NodeMetadata)>,
	},
	PrioritiseStates(Vec<u32>),
	ResetPriority,
}

#[derive(Debug)]
pub enum IpcError {
	EndOfFile,
	Interrupted,
	PollingReceiveFragmented,
	PollingReceiveTooLarge,
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
