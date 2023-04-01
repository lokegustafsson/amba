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
	time::Duration,
};

use data_structures::GraphIpc;
use eframe::egui::Context;
use graphui::{EmbeddingParameters, Graph2D};
use ipc::{IpcError, IpcMessage};
use qmp_client::{QmpClient, QmpCommand, QmpError, QmpEvent};

use crate::{cmd::Cmd, gui::Model, run::session::S2EConfig, SessionConfig};

mod session;

pub enum ControllerMsg {
	GuiShutdown,
	QemuShutdown,
	TellQemuPid(u32),
	ReplaceGraph {
		symbolic_state_graph: GraphIpc,
		basic_block_graph: GraphIpc,
	},
}
pub struct Controller {
	pub tx: mpsc::Sender<ControllerMsg>,
	pub rx: mpsc::Receiver<ControllerMsg>,
	pub model: Arc<Model>,
	pub gui_context: Option<Context>,
	pub qemu_pid: Option<u32>,
	pub embedder_tx: Option<mpsc::Sender<[GraphIpc; 2]>>,
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
						&embedder_model.state_graph,
						&embedder_model.block_graph,
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
						.map(|tx| tx.send([symbolic_state_graph, basic_block_graph]));
					self.gui_context.as_ref().map(|ctx| ctx.request_repaint());
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

fn prepare_run(cmd: &mut Cmd, config: &SessionConfig) -> Result<(), ()> {
	fn data_dir_has_been_initialized(cmd: &mut Cmd, data_dir: &Path) -> bool {
		let version_file = &data_dir.join("version.txt");
		let version = version_file
			.exists()
			.then(|| String::from_utf8(cmd.read(version_file)).unwrap());
		let initialized = version.is_some() && !version.unwrap().is_empty();
		if !initialized {
			tracing::error!("$AMBA_DATA_DIR/version.txt is missing or empty");
		}
		initialized
	}

	if !data_dir_has_been_initialized(cmd, &config.base.data_dir) {
		tracing::error!(
			?config.base.data_dir,
			"AMBA_DATA_DIR has not been initialized"
		);
		return Err(());
	}

	if config.session_dir.exists() {
		tracing::error!(
			?config.session_dir,
			"session_dir already exists, are multiple amba instances running concurrently?"
		);
		return Err(());
	}
	if config.temp_dir.exists() {
		tracing::error!(
			?config.temp_dir,
			"temp_dir already exists. How did this happen?!"
		);
		return Err(());
	}
	cmd.create_dir_all(&config.session_dir);
	cmd.create_dir_all(&config.temp_dir);

	// Populate the `session_dir`
	S2EConfig::new(
		cmd,
		&config.session_dir,
		&config.recipe_path,
		&config.recipe,
	)
	.save_to(
		cmd,
		&config.base.dependencies_dir,
		&config.session_dir,
	);

	Ok(())
}

fn run_ipc(ipc_socket: &Path, controller_tx: mpsc::Sender<ControllerMsg>) -> Result<(), ()> {
	let ipc_listener = UnixListener::bind(&ipc_socket).unwrap();
	let stream = ipc_listener.accept().unwrap().0;
	let (_ipc_tx, mut ipc_rx) = ipc::new_wrapping(&stream);
	tracing::info!("IPC initialized");
	loop {
		match ipc_rx.blocking_receive() {
			Ok(IpcMessage::GraphSnapshot {
				symbolic_state_graph,
				basic_block_graph,
			}) => {
				controller_tx
					.send(ControllerMsg::ReplaceGraph {
						symbolic_state_graph: symbolic_state_graph.into_owned(),
						basic_block_graph: basic_block_graph.into_owned(),
					})
					.unwrap_or_else(|mpsc::SendError(_)| {
						tracing::warn!("ipc failed messaging controller: already shut down")
					});
			}
			Ok(msg) => tracing::info!(?msg),
			Err(IpcError::EndOfFile) => break,
			Err(other) => panic!("ipc error: {other:?}"),
		}
	}
	stream.shutdown(Shutdown::Both).unwrap();
	tracing::info!("IPC shut down");
	Ok(())
}

fn run_qemu(
	cmd: &mut Cmd,
	config: &SessionConfig,
	qmp_socket: &Path,
	controller_tx: mpsc::Sender<ControllerMsg>,
) -> Result<(), ()> {
	// supporting single- vs multi-path
	let s2e_mode = match true {
		true => "s2e",
		false => "s2e_sp",
	};
	let arch = "x86_64";

	let qemu = &config
		.base
		.dependencies_dir
		.join(format!("bin/qemu-system-{arch}"));
	let libs2e_dir = &config.base.dependencies_dir.join("share/libs2e");
	let libs2e = &libs2e_dir.join(format!("libs2e-{arch}-{s2e_mode}.so"));
	let s2e_config = &config.session_dir.join("s2e-config.lua");
	let max_processes = 1;
	let image = &config
		.base
		.data_dir
		.join("images/ubuntu-22.04-x86_64/image.raw.s2e");

	let status = run_qemu_inner(
		cmd,
		config.sigstop_before_qemu_exec,
		qemu,
		&config.temp_dir,
		libs2e,
		libs2e_dir,
		s2e_config,
		max_processes,
		image,
		qmp_socket,
		|pid| controller_tx.send(ControllerMsg::TellQemuPid(pid)).unwrap(),
	);
	match status.success() {
		true => Ok(()),
		false => {
			tracing::error!(?status, "qemu exited with error code");
			Err(())
		}
	}
}

fn run_qemu_inner(
	cmd: &mut Cmd,
	sigstop_qemu_on_fork: bool,
	qemu: &Path,
	temp_dir: &Path,
	libs2e: &Path,
	libs2e_dir: &Path,
	s2e_config: &Path,
	max_processes: u16,
	image: &Path,
	qmp_socket: &Path,
	with_pid: impl FnOnce(u32),
) -> ExitStatus {
	assert!(qemu.exists());
	assert!(libs2e.exists());
	assert!(s2e_config.exists());
	assert!(libs2e_dir.exists());

	let mut command = Command::new(qemu);
	command
		.current_dir(temp_dir)
		.env("LD_PRELOAD", libs2e)
		.env("S2E_CONFIG", s2e_config)
		.env("S2E_SHARED_DIR", libs2e_dir)
		.env("S2E_MAX_PROCESSES", max_processes.to_string())
		.env("S2E_UNBUFFERED_STREAM", "1");

	if sigstop_qemu_on_fork {
		// Before exec, and hence actually starting QEMU, the child process sends
		// SIGSTOP to itself. We can then start debugging by attaching to the QEMU pid
		// and sending SIGCONT
		// SAFETY: `raise` and `write` are async-safe. We do not allocate memory.
		unsafe {
			let mut buf = String::with_capacity(256);
			command.pre_exec(move || {
				use std::fmt::Write;

				let _ = writeln!(
					buf,
					"[pre-exec before SIGSTOP] stopped with pid={}",
					process::id()
				);

				nix::unistd::write(2, buf.as_ref())?;
				nix::sys::signal::raise(nix::sys::signal::Signal::SIGSTOP)?;
				nix::unistd::write(2, b"[pre-exec after SIGSTOP] resuming!\n")?;

				Ok(())
			});
		}
	}
	if max_processes > 1 {
		command.arg("-nographic");
	}
	command
		.arg("-qmp")
		.arg({
			let mut line = OsString::new();
			line.push("unix:");
			line.push(qmp_socket);
			line.push(",server,nowait");
			line
		})
		.arg("-drive")
		.arg({
			let mut line = OsString::new();
			line.push("file=");
			line.push(image);
			{
				let bytes = <OsStr as std::os::unix::ffi::OsStrExt>::as_bytes(image.as_ref());
				assert!(!bytes.contains(&b','));
			}
			line.push(",format=s2e,cache=writeback");
			line
		})
		.args([
			"-k",
			"en-us",
			"-monitor",
			"null",
			"-m",
			"256M",
			"-enable-kvm",
			"-serial",
			"file:/dev/stdout",
			"-net",
			"none",
			"-net",
			"nic,model=e1000",
			"-loadvm",
			"ready",
		]);
	cmd.command_spawn_wait_with_pid(&mut command, with_pid)
}

fn run_qmp(socket: &Path, controller_tx: mpsc::Sender<ControllerMsg>) -> Result<(), ()> {
	let stream = 'outer: {
		let mut attempt = 0;
		let result = loop {
			attempt += 1;
			match UnixStream::connect(socket) {
				Ok(stream) => break 'outer stream,
				Err(err) if attempt > 10 => break err,
				Err(_) => {}
			}
			thread::sleep(Duration::from_millis(50));
		};
		tracing::error!(?result, ?socket, "failed to connect to socket");
		return Err(());
	};

	let mut qmp = QmpClient::new(&stream);

	let event_handler = |event @ QmpEvent { .. }| {
		tracing::info!(?event, "QMP");
	};

	let greeting = qmp.blocking_receive().expect("greeting");
	tracing::info!(?greeting, "QMP");

	let negotiated = qmp
		.blocking_request(&QmpCommand::QmpCapabilities, event_handler)
		.unwrap();
	tracing::info!(?negotiated, "QMP");

	let status = qmp
		.blocking_request(&QmpCommand::QueryStatus, event_handler)
		.unwrap();
	tracing::info!(?status, "QMP");

	loop {
		match qmp.blocking_receive() {
			Ok(response) => {
				tracing::info!(?response, "QMP");
				if let qmp_client::QmpResponse::Event(QmpEvent { event, .. }) = response {
					if event == "SHUTDOWN" {
						controller_tx.send(ControllerMsg::QemuShutdown).unwrap();
						return Ok(());
					}
				}
			}
			Err(QmpError::EndOfFile) => return Err(()),
			Err(err) => unreachable!("{:?}", err),
		}
	}
}

fn run_embedder(
	state_graph: &RwLock<Graph2D>,
	block_graph: &RwLock<Graph2D>,
	embedding_parameters: &Mutex<EmbeddingParameters>,
	rx: mpsc::Receiver<[GraphIpc; 2]>,
	gui_context: Option<Context>,
) -> Result<(), ()> {
	// TODO Non-zero timeout once really stable, to save cpu
	loop {
		let params = embedding_parameters.lock().unwrap().clone();
		match rx.try_recv() {
			Ok([state, block]) => {
				*state_graph.write().unwrap() = Graph2D::new(state, params);
				*block_graph.write().unwrap() = Graph2D::new(block, params);
				continue;
			}
			Err(mpsc::TryRecvError::Empty) => {}
			Err(mpsc::TryRecvError::Disconnected) => {
				tracing::info!("exiting");
				return Ok(());
			}
		}
		for graph in [state_graph, block_graph] {
			let mut working_copy = graph.read().unwrap().clone();
			working_copy.run_layout_iterations(100, params);
			*graph.write().unwrap() = working_copy;
		}
		gui_context.as_ref().map(|ctx| ctx.request_repaint());
	}
}
