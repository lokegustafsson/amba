//! The Gui controller

use std::{
	mem,
	net::Shutdown,
	os::unix::net::UnixStream,
	path::Path,
	sync::{mpsc, Arc},
	thread::{self, ScopedJoinHandle},
};

use eframe::egui::Context;
use ipc::GraphIpc;

use crate::{
	cmd::Cmd,
	gui::Model,
	run::{embed, run},
	SessionConfig,
};

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

pub enum EmbedderMsg {
	ReplaceGraph([GraphIpc; 2]),
	WakeUp,
}

pub struct Controller {
	pub tx: mpsc::Sender<ControllerMsg>,
	pub rx: mpsc::Receiver<ControllerMsg>,
	pub model: Arc<Model>,
	pub gui_context: Option<Context>,
	pub qemu_pid: Option<u32>,
	pub embedder_tx: Option<mpsc::Sender<EmbedderMsg>>,
}
impl Controller {
	/// Launch QEMU+S2E. That is, we do the equivalent of
	/// <https://github.com/S2E/s2e-env/blob/master/s2e_env/templates/launch-s2e.sh>
	/// but in rust code.
	///
	/// TODO: support more guests than just `ubuntu-22.04-x86_64`
	pub fn run(mut self, cmd: &mut Cmd, config: &SessionConfig) -> Result<(), ()> {
		run::prepare_run(cmd, config)?;

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
				.spawn_scoped(s, || {
					run::run_ipc(ipc_socket, controller_tx_from_ipc)
				})
				.unwrap();
			let qemu = thread::Builder::new()
				.name("qemu".to_owned())
				.spawn_scoped(s, || {
					run::run_qemu(cmd, config, qmp_socket, controller_tx_from_qemu)
				})
				.unwrap();
			let qmp = thread::Builder::new()
				.name("qmp".to_owned())
				.spawn_scoped(s, || {
					run::run_qmp(qmp_socket, controller_tx_from_qmp)
				})
				.unwrap();
			let embedder = thread::Builder::new()
				.name("embedder".to_owned())
				.spawn_scoped(s, || {
					embed::run_embedder(&embedder_model, embedder_rx, embedder_gui_context)
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
					self.embedder_tx.as_ref().map(|tx| {
						tx.send(EmbedderMsg::ReplaceGraph([
							symbolic_state_graph,
							basic_block_graph,
						]))
					});
				}
				ControllerMsg::EmbeddingParamsUpdated => {
					if let Some(tx) = self.embedder_tx.as_ref() {
						let (Ok(_) | Err(_)) = tx.send(EmbedderMsg::WakeUp);
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
