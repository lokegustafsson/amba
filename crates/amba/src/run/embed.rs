//! The worker thread for the gui

use std::{sync::mpsc, thread};

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
		&config.executable_host_path(),
		config.recipe_path.parent().unwrap(),
	)
	.unwrap();
	let mut thread_pool_size = (thread::available_parallelism().unwrap().get() / 2).max(1);
	let mut thread_pool = rayon::ThreadPoolBuilder::new()
		.num_threads(thread_pool_size)
		.build()
		.unwrap();
	loop {
		let message = if blocking {
			// Will wait
			rx.recv().map_err(Into::into)
		} else {
			// Will merely check if there are pending messages
			rx.try_recv()
		};
		match message {
			Ok(EmbedderMsg::UpdateEdges {
				block_edges,
				state_edges,
			}) => {
				model.add_new_edges(state_edges, block_edges, &mut disasm_context);

				blocking = false;
				continue;
			}
			Ok(EmbedderMsg::WakeUp) => {
				blocking = false;
				continue;
			}
			Ok(EmbedderMsg::QemuShutdown) => {
				let new_thread_pool_size = thread::available_parallelism().unwrap().get();
				if new_thread_pool_size != thread_pool_size {
					thread_pool_size = new_thread_pool_size;
					thread_pool = rayon::ThreadPoolBuilder::new()
						.num_threads(thread_pool_size)
						.build()
						.unwrap();
				}
			}
			Err(mpsc::TryRecvError::Empty) => {}
			Err(mpsc::TryRecvError::Disconnected) => {
				tracing::info!("exiting");
				return Ok(());
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
