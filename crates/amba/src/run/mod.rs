//! The run subcommand

#![allow(unsafe_code)]

use std::{
	ffi::{OsStr, OsString},
	mem,
	net::Shutdown,
	os::unix::{
		net::{UnixListener, UnixStream},
		process::CommandExt,
	},
	path::Path,
	process::{self, Command, ExitStatus},
	sync::{mpsc, Arc, Mutex, RwLock},
	thread::{self, ScopedJoinHandle},
	time::{Duration, Instant},
};

use eframe::egui::Context;
use graphui::{EmbeddingParameters, Graph2D};
use ipc::{GraphIpc, IpcError, IpcMessage};
use qmp_client::{QmpClient, QmpCommand, QmpError, QmpEvent};

use crate::{cmd::Cmd, gui::Model, run::session::S2EConfig, SessionConfig};

mod session;
mod run;
mod control;

fn run_embedder(
	state_graph: &RwLock<Graph2D>,
	block_graph: &RwLock<Graph2D>,
	embedding_parameters: &Mutex<EmbeddingParameters>,
	rx: mpsc::Receiver<Option<[GraphIpc; 2]>>,
	gui_context: Option<Context>,
) -> Result<(), ()> {
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
				*state_graph.write().unwrap() = Graph2D::new(state, start_params);
				*block_graph.write().unwrap() = Graph2D::new(block, start_params);
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

		for graph in [state_graph, block_graph] {
			let mut working_copy = graph.read().unwrap().clone();
			total_delta_pos += working_copy.run_layout_iterations(iterations, params);
			*graph.write().unwrap() = working_copy;
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
