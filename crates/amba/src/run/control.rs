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
use ipc::{IpcInstance, IpcTx, NodeMetadata};
use model::Model;

use crate::{
	cmd::Cmd,
	run::{embed, runners},
	SessionConfig,
};

pub enum ControllerMsg {
	GuiShutdown,
	QemuShutdown,
	TellQemuPid(u32),
	UpdateEdges {
		block_edges: Vec<(NodeMetadata, NodeMetadata)>,
		state_edges: Vec<(NodeMetadata, NodeMetadata)>,
	},
	EmbeddingParamsOrViewUpdated,
	NewPriority(usize),
}

pub enum EmbedderMsg {
	UpdateEdges {
		block_edges: Vec<(NodeMetadata, NodeMetadata)>,
		state_edges: Vec<(NodeMetadata, NodeMetadata)>,
	},
	WakeUp,
}

pub struct Controller {
	pub tx: mpsc::Sender<ControllerMsg>,
	pub rx: mpsc::Receiver<ControllerMsg>,
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
	pub fn run(
		mut self,
		cmd: &mut Cmd,
		config: &SessionConfig,
		model: Arc<Model>,
	) -> Result<(), ()> {
		runners::prepare_run(cmd, config)?;

		let ipc_socket = &config.temp_dir.join("amba-ipc.socket");
		let qmp_socket = &config.temp_dir.join("qmp.socket");
		let controller_tx_from_ipc = self.tx.clone();
		let controller_tx_from_qemu = self.tx.clone();
		let controller_tx_from_qmp = self.tx.clone();
		let (embedder_tx, embedder_rx) = mpsc::channel();
		self.embedder_tx = Some(embedder_tx);
		let embedder_gui_context = self.gui_context.clone();

		let res = thread::scope(|s| {
			let qemu = thread::Builder::new()
				.name("qemu".to_owned())
				.spawn_scoped(s, || {
					runners::run_qemu(cmd, config, qmp_socket, controller_tx_from_qemu)
				})
				.unwrap();
			let ipc_instance = IpcInstance::new_gui(ipc_socket);
			let (ipc_rx, ipc_tx) = ipc_instance.into();
			let ipc = thread::Builder::new()
				.name("ipc".to_owned())
				.spawn_scoped(s, || {
					runners::run_ipc(ipc_rx, controller_tx_from_ipc)
				})
				.unwrap();
			let qmp = thread::Builder::new()
				.name("qmp".to_owned())
				.spawn_scoped(s, || {
					runners::run_qmp(qmp_socket, controller_tx_from_qmp)
				})
				.unwrap();
			let embedder_model = model.clone();
			let embedder = thread::Builder::new()
				.name("embedder".to_owned())
				.spawn_scoped(s, move || {
					embed::run_embedder(
						&embedder_model,
						embedder_rx,
						embedder_gui_context,
						config,
					)
				})
				.unwrap();
			self.run_controller(ipc_tx, model);
			self.shutdown_controller(ipc_socket, ipc, qemu, qmp, embedder)
		});
		cmd.try_remove(ipc_socket);
		cmd.try_remove(qmp_socket);
		res
	}

	fn run_controller(&mut self, mut ipc_tx: IpcTx, model: Arc<Model>) {
		loop {
			match self.rx.recv().unwrap() {
				ControllerMsg::GuiShutdown => return,
				ControllerMsg::QemuShutdown => {
					if self.gui_context.is_none() {
						return;
					}
				}
				ControllerMsg::TellQemuPid(pid) => self.qemu_pid = Some(pid),
				ControllerMsg::UpdateEdges {
					block_edges,
					state_edges,
				} => {
					self.embedder_tx.as_ref().map(|tx| {
						tx.send(EmbedderMsg::UpdateEdges {
							block_edges,
							state_edges,
						})
					});
				}
				ControllerMsg::EmbeddingParamsOrViewUpdated => {
					if let Some(tx) = self.embedder_tx.as_ref() {
						let (Ok(_) | Err(_)) = tx.send(EmbedderMsg::WakeUp);
					}
				}
				ControllerMsg::NewPriority(prio) => {
					let states = model.as_ref().get_neighbour_states(prio);

					tracing::info!("Sending state prio: {states:#?}");

					let msg_result =
						ipc_tx.blocking_send(&ipc::IpcMessage::PrioritiseStates(states));
					if msg_result.is_err() {
						tracing::info!("State priority signal sent, but execution has completed");
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
