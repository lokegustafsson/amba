use std::{
	io::{self, BufReader, BufWriter, Read, Write},
	time::{Duration, SystemTime},
};

use read_until::ReadExt;
use serde::{
	de::Deserializer,
	ser::{SerializeStruct, Serializer},
	Deserialize, Serialize,
};
use serde_json::{Map, Value};

mod read_until;

/// A QMP (QEMU Machine Protocol) client communicating over the provided stream.
/// The stream does not need to be buffered as the `QmpClient` buffers
/// internally.
pub struct QmpClient<S: Copy + Read + Write> {
	stream_tx: BufWriter<S>,
	stream_rx: BufReader<S>,
	id: u64,
}

impl<S: Copy + Read + Write> QmpClient<S> {
	pub fn new(stream: S) -> Self {
		Self {
			stream_tx: BufWriter::new(stream),
			stream_rx: BufReader::new(stream),
			id: 1,
		}
	}

	pub fn blocking_send<T: Serialize>(&mut self, request: &QmpRequest<T>) {
		serde_json::to_writer(&mut self.stream_tx, request).unwrap();
		self.stream_tx.write_all(b"\n").unwrap();
		self.stream_tx.flush().unwrap();
	}

	pub fn blocking_receive(&mut self) -> Result<QmpResponse, QmpError> {
		Ok(serde_json::from_reader(self.stream_rx.take_until(b'\n')).unwrap())
	}

	pub fn blocking_request<F: FnMut(QmpEvent)>(
		&mut self,
		command: &QmpCommand,
		mut event_handler: F,
	) -> Result<QmpResponse, QmpError> {
		self.blocking_send(&QmpRequest {
			asynchronous: false,
			command: command.get_command(),
			arguments: command.get_arguments(),
			id: self.id,
		});
		self.id += 1;
		loop {
			match self.blocking_receive()? {
				QmpResponse::Event(event) => event_handler(event),
				other => return Ok(other),
			}
		}
	}
}

#[derive(Debug)]
pub enum QmpError {
	EndOfFile,
	Interrupted,
	Io(io::Error),
}

#[derive(Debug)]
pub struct QmpRequest<T: Serialize> {
	/// NOTE: Only some requests can be done asynchronously ("out of bounds")
	pub asynchronous: bool,
	pub command: &'static str,
	pub arguments: Option<T>,
	pub id: u64,
}

impl<T: Serialize> Serialize for QmpRequest<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct(
			"QmpRequest",
			match self.arguments.is_some() {
				true => 3,
				false => 2,
			},
		)?;
		state.serialize_field(
			if self.asynchronous {
				"exec-oob"
			} else {
				"execute"
			},
			self.command,
		)?;
		if let Some(arguments) = &self.arguments {
			state.serialize_field("arguments", arguments)?;
		}
		state.serialize_field("id", &self.id)?;
		state.end()
	}
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum QmpResponse {
	Greeting {
		#[serde(rename = "QMP")]
		qmp: QemuGreeting,
	},
	Response {
		#[serde(rename = "return")]
		ret: Value,
		id: u64,
	},
	Error {
		error: QemuError,
		id: u64,
	},
	Event(QmpEvent),
}

#[derive(Deserialize, Debug)]
pub struct QemuGreeting {
	pub version: QemuVersion,
	pub capabilities: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct QemuVersion {
	pub qemu: QemuVersionCode,
	pub package: String,
}

#[derive(Deserialize, Debug)]
pub struct QemuVersionCode {
	pub major: u16,
	pub minor: u16,
	pub micro: u16,
}

#[derive(Deserialize, Debug)]
pub struct QemuError {
	pub class: String,
	pub desc: String,
}

#[derive(Deserialize, Debug)]
pub struct QmpEvent {
	pub event: String,
	pub data: Map<String, Value>,
	#[serde(deserialize_with = "de_qmp_systemtime")]
	pub timestamp: SystemTime,
}

fn de_qmp_systemtime<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SystemTime, D::Error> {
	#[derive(Deserialize)]
	struct Repr {
		seconds: u64,
		microseconds: u64,
	}
	let repr: Repr = Repr::deserialize(deserializer)?;
	Ok(SystemTime::UNIX_EPOCH
		.checked_add(Duration::from_secs(repr.seconds) + Duration::from_micros(repr.microseconds))
		.unwrap())
}

pub enum QmpCommand {
	QmpCapabilities,
	QueryStatus,
	Screendump { filename: String },
	Stop,
	Cont,
}

impl QmpCommand {
	fn get_command(&self) -> &'static str {
		match self {
			Self::QmpCapabilities => "qmp_capabilities",
			Self::QueryStatus => "query-status",
			Self::Screendump { .. } => "screendump",
			Self::Stop => "stop",
			Self::Cont => "cont",
		}
	}

	fn get_arguments(&self) -> Option<Value> {
		match self {
			Self::Screendump { filename } => Some(serde_json::json!({ filename: filename })),
			_ => None,
		}
	}
}
