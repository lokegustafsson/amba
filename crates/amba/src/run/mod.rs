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

pub enum ControllerMsg {
	GuiShutdown,
	QemuShutdown,
	TellQemuPid(u32),
	ReplaceGraph {
		symbolic_state_graph: GraphIpc,
		basic_block_graph: GraphIpc,
	},
	EmbeddingParamsUpdated,
}
pub struct Controller {
	pub tx: mpsc::Sender<ControllerMsg>,
	pub rx: mpsc::Receiver<ControllerMsg>,
	pub model: Arc<Model>,
	pub gui_context: Option<Context>,
	pub qemu_pid: Option<u32>,
	pub embedder_tx: Option<mpsc::Sender<Option<[GraphIpc; 2]>>>,
}
impl Controller {
	/// Launch QEMU+S2E. That is, we do the equivalent of
	/// <https://github.com/S2E/s2e-env/blob/master/s2e_env/templates/launch-s2e.sh>
	/// but in rust code.
	///
	/// TODO: support more guests than just `ubuntu-22.04-x86_64`
	pub fn run(mut self, cmd: &mut Cmd, config: &SessionConfig) -> Result<(), ()> {
		prepare_run(cmd, config)?;

		let ipc_socket = &config.temp_dir.join("amba-ipc.socket");
		let qmp_socket = &config.temp_dir.join("qmp.socket");
		let controller_tx_from_ipc = self.tx.clone();
		let controller_tx_from_qemu = self.tx.clone();
		let controller_tx_from_qmp = self.tx.clone();
		let embedder_model = self.model.clone();
		let (embedder_tx, embedder_rx) = mpsc::channel();
		self.embedder_tx = Some(embedder_tx);
		let embedder_gui_context = self.gui_context.clone();

		let res = thread::scope(|s| {
			let ipc = thread::Builder::new()
				.name("ipc".to_owned())
				.spawn_scoped(s, || run_ipc(ipc_socket, controller_tx_from_ipc))
				.unwrap();
			let qemu = thread::Builder::new()
				.name("qemu".to_owned())
				.spawn_scoped(s, || {
					run_qemu(cmd, config, qmp_socket, controller_tx_from_qemu)
				})
				.unwrap();
			let qmp = thread::Builder::new()
				.name("qmp".to_owned())
				.spawn_scoped(s, || run_qmp(qmp_socket, controller_tx_from_qmp))
				.unwrap();
			let embedder = thread::Builder::new()
				.name("embedder".to_owned())
				.spawn_scoped(s, || {
					run_embedder(
						&embedder_model.drawable_state_graph,
						&embedder_model.drawable_block_graph,
						&embedder_model.embedding_parameters,
						embedder_rx,
						embedder_gui_context,
					)
				})
				.unwrap();
			self.run_controller();
			self.shutdown_controller(ipc_socket, ipc, qemu, qmp, embedder)
		});
		cmd.try_remove(ipc_socket);
		cmd.try_remove(qmp_socket);
		res
	}

	fn run_controller(&mut self) {
		loop {
			match self.rx.recv().unwrap() {
				ControllerMsg::GuiShutdown => return,
				ControllerMsg::QemuShutdown => {
					if self.gui_context.is_none() {
						return;
					}
				}
				ControllerMsg::TellQemuPid(pid) => self.qemu_pid = Some(pid),
				ControllerMsg::ReplaceGraph {
					symbolic_state_graph,
					basic_block_graph,
				} => {
					self.embedder_tx
						.as_ref()
						.map(|tx| tx.send(Some([symbolic_state_graph, basic_block_graph])));
					if let Some(ctx) = self.gui_context.as_ref() {
						ctx.request_repaint();
					}
				}
				ControllerMsg::EmbeddingParamsUpdated => {
					if let Some(tx) = self.embedder_tx.as_ref() {
						let (Ok(_) | Err(_)) = tx.send(None);
					}
				}
			}
		}
	}

	fn shutdown_controller(
		self,
		ipc_socket: &Path,
		ipc: ScopedJoinHandle<'_, Result<(), ()>>,
		qemu: ScopedJoinHandle<'_, Result<(), ()>>,
		qmp: ScopedJoinHandle<'_, Result<(), ()>>,
		embedder: ScopedJoinHandle<'_, Result<(), ()>>,
	) -> Result<(), ()> {
		match UnixStream::connect(ipc_socket) {
			Ok(conn) => conn.shutdown(Shutdown::Both).unwrap(),
			Err(_) => {}
		}
		self.qemu_pid.map(|pid| {
			nix::sys::signal::kill(
				nix::unistd::Pid::from_raw(pid.try_into().unwrap()),
				Some(nix::sys::signal::Signal::SIGTERM),
			)
		});
		mem::drop(self.embedder_tx);
		qmp.join().unwrap()?;
		qemu.join().unwrap()?;
		ipc.join().unwrap()?;
		embedder.join().unwrap()?;
		Ok(())
	}
}

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
