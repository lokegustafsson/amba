//! The run subcommand

#![allow(unsafe_code)]

use std::{sync::mpsc, time::Instant};

use eframe::egui::Context;
use graphui::Graph2D;
use ipc::GraphIpc;

use crate::gui::Model;

pub fn run_embedder(
	model: &Model,
	rx: mpsc::Receiver<Option<[GraphIpc; 2]>>,
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
			Ok(Some([state, block])) => {
				let mut start_params = params;
				start_params.noise = 0.1;
				*raw_state_graph.write().unwrap() = Graph2D::new(state.clone(), start_params);
				*raw_block_graph.write().unwrap() = Graph2D::new(block.clone(), start_params);
				*compressed_state_graph.write().unwrap() = Graph2D::new(state, start_params);
				*compressed_block_graph.write().unwrap() = Graph2D::new(block, start_params);
				blocking = false;
				continue;
			}
			Ok(None) => {
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
