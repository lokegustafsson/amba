//! The worker thread for the gui

use std::{sync::mpsc, time::Instant};

use data_structures::ControlFlowGraph;
use eframe::egui::Context;
use graphui::Graph2D;

use crate::{gui::Model, run::control::EmbedderMsg};

pub fn run_embedder(
	model: &Model,
	rx: mpsc::Receiver<EmbedderMsg>,
	gui_context: Option<Context>,
) -> Result<(), ()> {
	let Model {
		raw_state_graph,
		raw_block_graph,
		compressed_state_graph,
		compressed_block_graph,
		embedding_parameters,
	} = model;

	let mut updates_per_second: f64 = 0.0;
	let iterations = 100;
	let mut blocking = true;
	loop {
		let params = {
			let mut guard = embedding_parameters.lock().unwrap();
			guard.statistic_updates_per_second = iterations as f64 * updates_per_second;
			*guard
		};
		match blocking
			.then(|| rx.recv().map_err(Into::into))
			.unwrap_or_else(|| rx.try_recv())
		{
			Ok(EmbedderMsg::ReplaceGraph([state, block])) => {
				let mut start_params = params;
				start_params.noise = 0.1;
				let compressed_state = {
					let g: ControlFlowGraph = (&state).into();
					g.compressed_graph.into()
				};
				let compressed_block = {
					let g: ControlFlowGraph = (&block).into();
					g.compressed_graph.into()
				};
				*raw_state_graph.write().unwrap() = Graph2D::new(state, start_params);
				*raw_block_graph.write().unwrap() = Graph2D::new(block, start_params);
				*compressed_state_graph.write().unwrap() =
					Graph2D::new(compressed_state, start_params);
				*compressed_block_graph.write().unwrap() =
					Graph2D::new(compressed_block, start_params);
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
		let timer = Instant::now();
		let mut total_delta_pos = 0.0;

		for graph in [
			raw_state_graph,
			raw_block_graph,
			compressed_block_graph,
			compressed_state_graph,
		] {
			let mut graph_lock = graph.write().unwrap();
			let mut working_copy = graph_lock.clone();
			total_delta_pos += working_copy.run_layout_iterations(iterations, params);
			*graph_lock = working_copy;
		}
		updates_per_second = timer.elapsed().as_secs_f64().recip();
		if total_delta_pos < 0.1 {
			updates_per_second = 0.0;
			blocking = true;
		}

		if let Some(ctx) = gui_context.as_ref() {
			ctx.request_repaint();
		}
	}
}
