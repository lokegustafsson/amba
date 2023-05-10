//! The worker thread for the gui

use std::sync::mpsc;

use disassembler::DebugInfoContext;
use eframe::egui::Context;
use model::{LayoutMadeProgress, Model};

use crate::{run::control::EmbedderMsg, SessionConfig};

pub fn run_embedder(
	model: &Model,
	rx: mpsc::Receiver<EmbedderMsg>,
	gui_context: Option<Context>,
	config: &SessionConfig,
) -> Result<(), ()> {
	let mut blocking = true;
	let debug_info_context = DebugInfoContext::new(&config.executable_host_path()).unwrap();
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
				model.add_new_edges(state_edges, block_edges);

				blocking = false;
				continue;
			}
			Ok(EmbedderMsg::WakeUp) => {
				blocking = false;
				continue;
			}
			Err(mpsc::TryRecvError::Empty) => {}
			Err(mpsc::TryRecvError::Disconnected) => {
				tracing::info!("exiting");
				return Ok(());
			}
		}
		match model.run_layout_iterations() {
			LayoutMadeProgress::YesALot | LayoutMadeProgress::YesALittle => {}
			LayoutMadeProgress::NoJustTiny => blocking = true,
		}

		if let Some(ctx) = gui_context.as_ref() {
			ctx.request_repaint();
		}
	}
}
