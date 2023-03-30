use std::{
	borrow::Cow,
	ffi::CStr,
	num::{NonZeroU64, NonZeroUsize},
	os::unix::net::UnixStream,
	ptr::NonNull,
	sync::{
		atomic::{AtomicBool, Ordering},
		Mutex,
	},
	thread,
	time::Duration,
};

use data_structures::GraphIpc;
use ipc::{GraphKind, IpcError, IpcMessage, IpcTx};

use crate::control_flow::ControlFlowGraph;

pub mod control_flow;

static STATE: Mutex<Option<State>> = Mutex::new(None);
static STATE_SHUTDOWN: AtomicBool = AtomicBool::new(false);

struct State {
	ipc_tx: IpcTx<'static>,
	symbolic_state_graph: ControlFlowGraph,
	basic_block_graph: ControlFlowGraph,
	latest_executed_basic_block_vaddr: u64,
}
impl State {
	fn init() {
		let mut slot = STATE.lock().unwrap();
		assert!(slot.is_none());

		*slot = Some(Self {
			ipc_tx: match UnixStream::connect("amba-ipc.socket") {
				Ok(stream) => {
					let stream = Box::leak(Box::new(stream));
					let (tx, mut rx) = ipc::new_wrapping(&*stream);
					thread::spawn(move || loop {
						match rx.blocking_receive() {
							Ok(_) => println!("libamba received ipc message"),
							Err(IpcError::EndOfFile) => return,
							Err(other) => panic!("ipc error: {other:?}"),
						}
					});
					tx
				}
				Err(err) => panic!("libamba failed to connect to IPC socket: {err:?}"),
			},
			symbolic_state_graph: ControlFlowGraph::new(),
			basic_block_graph: ControlFlowGraph::new(),
			latest_executed_basic_block_vaddr: 0,
		});
		thread::spawn(|| loop {
			thread::sleep(Duration::from_millis(100));
			assert!(!STATE_SHUTDOWN.load(Ordering::SeqCst));

			let mut guard = STATE.lock().unwrap();
			let state = guard.as_mut().unwrap();
			state
				.ipc_tx
				.blocking_send(&IpcMessage::GraphSnapshot {
					kind: GraphKind::SymbolicStates,
					graph: Cow::Owned(GraphIpc::from(&state.symbolic_state_graph.graph)),
				})
				.unwrap_or_else(|err| {
					println!("libamba ipc error sending symbolic graph: {err:?}")
				});
			state
				.ipc_tx
				.blocking_send(&IpcMessage::GraphSnapshot {
					kind: GraphKind::BasicBlocks,
					graph: Cow::Owned(GraphIpc::from(&state.basic_block_graph.graph)),
				})
				.unwrap_or_else(|err| println!("libamba ipc error sending block graph: {err:?}"));
		});
	}

	fn shutdown(&mut self) {
		println!("Basic block graph\n{}", self.basic_block_graph);
		println!(
			"Symbolic state graph\n{}",
			self.symbolic_state_graph
		);
	}

	fn on_state_fork(&mut self, old_state_id: u32, new_state_ids: &[u32]) {
		for &new_state_id in new_state_ids {
			self.symbolic_state_graph
				.update(u64::from(old_state_id), u64::from(new_state_id));
		}
	}

	fn on_state_merge(&mut self, base_state_id: u32, other_state_id: u32) {
		self.symbolic_state_graph.update(
			u64::from(other_state_id),
			u64::from(base_state_id),
		);
	}

	fn on_translate_block(
		&mut self,
		block_virtual_addr: u64,
		block: Option<&[u8]>,
		module_path: Option<&CStr>,
		module_internal_offset: Option<NonZeroU64>,
	) {
		println!("Translating block at {block_virtual_addr} (raw: {block:?}) at offset {module_internal_offset:?} within module {module_path:?}");
	}

	fn on_watched_block_start_execute(&mut self, block_virtual_addr: u64) {
		self.basic_block_graph.update(
			self.latest_executed_basic_block_vaddr,
			block_virtual_addr,
		);
		self.latest_executed_basic_block_vaddr = block_virtual_addr;
	}
}

#[allow(unsafe_code, clippy::missing_safety_doc)]
mod ffi {
	use super::*;

	#[no_mangle]
	pub extern "C" fn rust_init() {
		State::init();
	}

	#[no_mangle]
	pub extern "C" fn rust_on_engine_shutdown() {
		STATE.lock().unwrap().as_mut().unwrap().shutdown();
	}

	#[no_mangle]
	pub unsafe extern "C" fn rust_on_state_fork(
		old_state_id: u32,
		new_state_ids_ptr: NonNull<u32>,
		new_state_ids_count: NonZeroUsize,
	) {
		let new_state_ids: &[u32] = std::slice::from_raw_parts(
			new_state_ids_ptr.as_ptr(),
			new_state_ids_count.get(),
		);
		STATE
			.lock()
			.unwrap()
			.as_mut()
			.unwrap()
			.on_state_fork(old_state_id, new_state_ids);
	}

	#[no_mangle]
	pub extern "C" fn rust_on_state_merge(base_state_id: u32, other_state_id: u32) {
		STATE
			.lock()
			.unwrap()
			.as_mut()
			.unwrap()
			.on_state_merge(base_state_id, other_state_id);
	}

	#[no_mangle]
	/// A module is a file on disk, e.g. an ELF executable or library.
	pub unsafe extern "C" fn rust_on_translate_block(
		block_virtual_addr: u64,
		block_data: *const u8,
		block_len: usize,
		module_path_cstr: *const i8,
		module_internal_offset: u64,
	) {
		let block: Option<&[u8]> = Option::zip(
			NonNull::new(block_data as *mut u8),
			NonZeroUsize::new(block_len),
		)
		.map(|(data, len)| std::slice::from_raw_parts(data.as_ptr(), len.get()));
		let module_path_str =
			NonNull::new(module_path_cstr as *mut i8).map(|ptr| CStr::from_ptr(ptr.as_ptr()));

		STATE.lock().unwrap().as_mut().unwrap().on_translate_block(
			block_virtual_addr,
			block,
			module_path_str,
			NonZeroU64::new(module_internal_offset),
		);
	}

	#[no_mangle]
	pub extern "C" fn rust_on_watched_block_start_execute(block_virtual_addr: u64) {
		STATE
			.lock()
			.unwrap()
			.as_mut()
			.unwrap()
			.on_watched_block_start_execute(block_virtual_addr);
	}
}
