#![allow(unsafe_code, clippy::missing_safety_doc)]

use std::{pin::Pin, slice, sync::Mutex};

use ipc::{IpcInstance, IpcMessage};

use crate::node_metadata::NodeMetadataFFIPair;

#[no_mangle]
pub extern "C" fn rust_new_ipc() -> *mut Mutex<IpcInstance> {
	let instance = IpcInstance::new_plugin("amba-ipc.socket".as_ref());
	Box::into_raw(Box::new(Mutex::new(instance)))
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_ipc(ptr: *mut Mutex<IpcInstance>) {
	let _ = Box::from_raw(ptr);
}

#[no_mangle]
unsafe fn send_ipc_message(ipc: *mut Mutex<IpcInstance>, msg: &ipc::IpcMessage) {
	let mut lock = (*ipc).lock().unwrap();
	let (_, ipc_tx) = lock.get_rx_tx();
	ipc_tx
		.blocking_send(msg)
		.unwrap_or_else(|err| println!("libamba ipc error: {err:?}"));
}

#[no_mangle]
pub unsafe extern "C" fn rust_ipc_send_edges(
	ipc: *mut Mutex<IpcInstance>,
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
}

#[no_mangle]
pub unsafe extern "C" fn rust_ipc_receive_message(
	ipc: *mut Mutex<IpcInstance>,
	vec: *mut cxx::CxxVector<i32>,
) -> bool {
	let mut lock = (*ipc).lock().unwrap();
	let (ipc_rx, _) = lock.get_rx_tx();
	let message = match ipc_rx.polling_receive() {
		Ok(m) => m,
		Err(ipc::IpcError::EndOfFile) => {
			println!("GUI has shut down");
			return false;
		},
		Err(err) => panic!("{err:?}"),
	};
	let res = match message {
		Some(IpcMessage::PrioritiseStates(states)) => {
			for state in states.into_iter() {
				Pin::new_unchecked(&mut *vec).push(state);
			}
			true
		}
		Some(IpcMessage::ResetPriority) | None => false,

		Some(_) => {
			panic!("Invalid IPC message")
		}
	};
	res
}
