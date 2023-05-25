//! The worker thread for the gui

use std::{collections::VecDeque, sync::mpsc, thread};

use disassembler::DisasmContext;
use eframe::egui::Context;
use graphui::EmbedderHasConverged;
use model::Model;

use crate::{run::control::EmbedderMsg, SessionConfig};

pub fn run_embedder(
	model: &Model,
	rx: mpsc::Receiver<EmbedderMsg>,
	gui_context: Option<Context>,
	config: &SessionConfig,
) -> Result<(), ()> {
	let mut blocking = true;
	let mut disasm_context = DisasmContext::new(
		config.executable_host_path().as_ref().map(AsRef::as_ref),
		config.recipe_path.parent().unwrap(),
	)
	.unwrap();
	let mut thread_pool_size = (thread::available_parallelism().unwrap().get() / 2).max(1);
	let mut thread_pool = rayon::ThreadPoolBuilder::new()
		.num_threads(thread_pool_size)
		.build()
		.unwrap();
	let mut unhandled_messages = VecDeque::new();
	loop {
		// Poll available messages
		loop {
			match rx.try_recv() {
				Ok(msg) => unhandled_messages.push_back(msg),
				Err(mpsc::TryRecvError::Empty) => break,
				Err(mpsc::TryRecvError::Disconnected) => {
					tracing::info!("exiting");
					return Ok(());
				}
			}
		}
		// Block awaiting message if none unhandled and `blocking = true`
		if blocking && unhandled_messages.is_empty() {
			match rx.recv() {
				Ok(msg) => unhandled_messages.push_back(msg),
				Err(mpsc::RecvError) => {
					tracing::info!("exiting");
					return Ok(());
				}
			}
		}
		// Handle messages
		while let Some(message) = unhandled_messages.pop_front() {
			match message {
				EmbedderMsg::UpdateEdges {
					mut block_edges,
					mut state_edges,
				} => {
					let mut update_chunk_count = 1;
					// Append additional sequential `EmbedderMsg::UpdateEdges`
					while let Some(EmbedderMsg::UpdateEdges {
						block_edges: block_extra,
						state_edges: state_extra,
					}) = unhandled_messages.front()
					{
						block_edges.extend_from_slice(&block_extra);
						state_edges.extend_from_slice(&state_extra);
						unhandled_messages.pop_front();
						update_chunk_count += 1;
					}
					model.add_new_edges(state_edges, block_edges, &mut disasm_context);
					tracing::info!("Registered edges from {update_chunk_count} IPC messages");

					blocking = false;
					continue;
				}
				EmbedderMsg::WakeUp => {
					blocking = false;
					continue;
				}
				EmbedderMsg::QemuShutdown => {
					let new_thread_pool_size = thread::available_parallelism().unwrap().get();
					if new_thread_pool_size != thread_pool_size {
						thread_pool_size = new_thread_pool_size;
						thread_pool = rayon::ThreadPoolBuilder::new()
							.num_threads(thread_pool_size)
							.build()
							.unwrap();
					}
				}
			}
		}
		match thread_pool.install(|| model.run_layout_iterations()) {
			EmbedderHasConverged::Yes => blocking = true,
			EmbedderHasConverged::No => {}
		}

		if let Some(ctx) = gui_context.as_ref() {
			ctx.request_repaint();
		}
	}
}
