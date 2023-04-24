#![allow(unsafe_code, clippy::missing_safety_doc)]

use std::{borrow::Cow, ffi::CStr, os::unix::net::UnixStream, pin::Pin, slice, sync::Mutex};

use ipc::{IpcMessage, IpcRx, IpcTx};

use crate::node_metadata::NodeMetadataFFIPair;

#[repr(C)]
pub struct IpcPair<'a> {
	tx: *mut Mutex<IpcTx<'a>>,
	rx: *mut Mutex<IpcRx<'a>>,
}

#[no_mangle]
pub extern "C" fn rust_new_ipc<'a>() -> IpcPair<'a> {
	let (tx, rx) = match UnixStream::connect("amba-ipc.socket") {
		Ok(stream) => {
			let stream = Box::leak(Box::new(stream));
			ipc::new_wrapping(&*stream)
		}
		Err(err) => panic!("libamba failed to connect to IPC socket: {err:?}"),
	};
	IpcPair {
		tx: Box::into_raw(Box::new(Mutex::new(tx))),
		rx: Box::into_raw(Box::new(Mutex::new(rx))),
	}
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_ipc(ptr: IpcPair<'_>) {
	let _ = Box::from_raw(ptr.tx);
	let _ = Box::from_raw(ptr.rx);
}

#[no_mangle]
pub unsafe extern "C" fn rust_ipc_send_edges(
	ipc: *mut Mutex<IpcTx<'_>>,
	state_data: *const NodeMetadataFFIPair,
	state_len: u64,
	block_data: *const NodeMetadataFFIPair,
	block_len: u64,
) {
	let state_edges = slice::from_raw_parts(state_data, state_len as _)
		.iter()
		.map(Into::into)
		.collect();
	let block_edges = slice::from_raw_parts(block_data, block_len as _)
		.iter()
		.map(Into::into)
		.collect();
	let msg = ipc::IpcMessage::NewEdges {
		state_edges,
		block_edges,
	};

	send_ipc_message(ipc, &msg);

	unsafe fn send_ipc_message(ipc: *mut Mutex<IpcTx<'_>>, msg: &ipc::IpcMessage) {
		let mut ipc = (*ipc).lock().unwrap();
		ipc.blocking_send(msg)
			.unwrap_or_else(|err| println!("libamba ipc error: {err:?}"));
	}
}

#[no_mangle]
pub unsafe extern "C" fn rust_ipc_receive_message(
	ipc: *mut Mutex<IpcRx<'_>>,
	vec: *mut cxx::CxxVector<u32>,
) -> bool {
	let mut ipc = (*ipc).lock().unwrap();
	let res = match ipc.blocking_receive() {
		Ok(IpcMessage::PrioritiseStates(states)) => {
			for state in states.into_iter() {
				Pin::new_unchecked(&mut *vec).push(state);
			}
			true
		}
		Ok(IpcMessage::ResetPriority) => false,
		Ok(_) => {
			panic!("Invalid IPC message");
		}
		Err(err) => {
			panic!("{err:?}");
		}
	};
	res
}
