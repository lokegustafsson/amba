//! The worker thread for the gui

use std::{sync::mpsc, time::Instant};

use eframe::egui::Context;

use crate::{gui::Model, run::control::EmbedderMsg};

pub fn run_embedder(
	model: &Model,
	rx: mpsc::Receiver<EmbedderMsg>,
	gui_context: Option<Context>,
) -> Result<(), ()> {
	let Model {
		block_control_flow,
		state_control_flow,
		raw_state_graph,
		raw_block_graph,
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
				let mut start_params = params;
				start_params.noise = 0.1;

				let mut block_control_flow = block_control_flow.write().unwrap();
				for (from, to) in block_edges.into_iter() {
					block_control_flow.update(from, to);
				}

				raw_block_graph
					.write()
					.unwrap()
					.into_new(block_control_flow.graph.clone(), start_params);
				compressed_block_graph.write().unwrap().into_new(
					block_control_flow.compressed_graph.clone(),
					start_params,
				);

				let mut state_control_flow = state_control_flow.write().unwrap();
				for (from, to) in state_edges.into_iter() {
					state_control_flow.update(from, to);
				}

				raw_state_graph
					.write()
					.unwrap()
					.into_new(state_control_flow.graph.clone(), start_params);

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

		for graph in [raw_state_graph, raw_block_graph, compressed_block_graph] {
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
