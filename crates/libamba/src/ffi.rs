use std::{os::unix::net::UnixStream, slice, sync::Mutex};

use ipc::IpcTx;

use crate::node_metadata::NodeMetadataFFIPair;

#[no_mangle]
pub extern "C" fn rust_new_ipc<'a>() -> *mut Mutex<IpcTx<'a>> {
	let ipc = match UnixStream::connect("amba-ipc.socket") {
		Ok(stream) => {
			let stream = Box::leak(Box::new(stream));
			let (tx, _rx) = ipc::new_wrapping(&*stream);
			tx
		}
		Err(err) => panic!("libamba failed to connect to IPC socket: {err:?}"),
	};
	Box::into_raw(Box::new(Mutex::new(ipc)))
}

#[no_mangle]
pub unsafe extern "C" fn rust_free_ipc(ptr: *mut Mutex<IpcTx<'_>>) {
	let _ = Box::from_raw(ptr);
}

unsafe fn send_ipc_message(ipc: *mut Mutex<IpcTx<'_>>, msg: &ipc::IpcMessage) {
	let mut ipc = (*ipc).lock().unwrap();
	ipc.blocking_send(msg)
		.unwrap_or_else(|err| println!("libamba ipc error: {err:?}"));
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
}

#[no_mangle]
pub extern "C" fn rust_main() -> std::ffi::c_int {
	println!("Hello world");
	0
}
