//! The run subcommand

#![allow(unsafe_code)]

use std::{
	ffi::{OsStr, OsString},
	net::Shutdown,
	os::unix::{
		net::{UnixListener, UnixStream},
		process::CommandExt,
	},
	path::Path,
	process::{self, Command, ExitStatus},
	thread,
	time::Duration,
};

use ipc::{Ipc, IpcError};
use qmp_client::{QmpClient, QmpCommand, QmpError, QmpEvent};
use recipe::{Recipe, RecipeError};

use crate::{cmd::Cmd, run::session::S2EConfig, RunArgs};

mod session;

/// Launch QEMU+S2E. That is, we do the equivalent of
/// <https://github.com/S2E/s2e-env/blob/master/s2e_env/templates/launch-s2e.sh>
/// but in rust code.
///
/// TODO: support more guests than just `ubuntu-22.04-x86_64`
pub fn run(
	cmd: &mut Cmd,
	dependencies_dir: &Path,
	data_dir: &Path,
	session_dir: &Path,
	temp_dir: &Path,
	RunArgs {
		recipe_path,
		debugger,
		qmp,
	}: RunArgs,
) -> Result<(), ()> {
	if !data_dir_has_been_initialized(cmd, data_dir) {
		tracing::error!(
			?data_dir,
			"AMBA_DATA_DIR has not been initialized"
		);
		return Err(());
	}

	if session_dir.exists() {
		tracing::error!(
			?session_dir,
			"session_dir already exists, are multiple amba instances running concurrently?"
		);
		return Err(());
	}
	if temp_dir.exists() {
		tracing::error!(
			?session_dir,
			"temp_dir already exists. How did this happen?!"
		);
		return Err(());
	}
	cmd.create_dir_all(session_dir);
	cmd.create_dir_all(temp_dir);
	// Populate the `session_dir`
	{
		let recipe = &match Recipe::deserialize_from(&cmd.read(&recipe_path)) {
			Ok(recipe) => recipe,
			Err(err) => {
				match err {
					RecipeError::NotRecipe(err) => {
						tracing::error!(?recipe_path, ?err, "Not a valid Recipe")
					}
					RecipeError::NotJson(err) => {
						tracing::error!(?recipe_path, ?err, "Not a valid JSON")
					}
					RecipeError::NotUtf8(err) => {
						tracing::error!(?recipe_path, ?err, "Not valid UTF8")
					}
				}
				return Err(());
			}
		};
		S2EConfig::new(cmd, session_dir, &recipe_path, recipe).save_to(
			cmd,
			dependencies_dir,
			session_dir,
		);
	}

	// supporting single- vs multi-path
	let s2e_mode = match true {
		true => "s2e",
		false => "s2e_sp",
	};
	let arch = "x86_64";

	let qemu = &dependencies_dir.join(format!("bin/qemu-system-{arch}"));
	let libs2e_dir = &dependencies_dir.join("share/libs2e");
	let libs2e = &libs2e_dir.join(format!("libs2e-{arch}-{s2e_mode}.so"));
	let s2e_config = &session_dir.join("s2e-config.lua");
	let max_processes = 1;
	let image = &data_dir.join("images/ubuntu-22.04-x86_64/image.raw.s2e");
	let serial_out = &session_dir.join("serial.txt");
	let qmp_socket = qmp.then(|| temp_dir.join("qmp.socket"));

	let ipc_socket = &temp_dir.join("amba-ipc.socket");
	let ipc_listener = UnixListener::bind(&ipc_socket).unwrap();

	let res = thread::scope(|s| {
		let ipc = s.spawn(|| {
			let stream = ipc_listener.accept().unwrap().0;
			let mut ipc = Ipc::new(&stream);
			loop {
				match ipc.blocking_receive() {
					Ok(msg) => tracing::info!(?msg),
					Err(IpcError::EndOfFile) => break,
					Err(other) => panic!("ipc error: {other:?}"),
				}
			}
			stream.shutdown(Shutdown::Both).unwrap();
		});
		let qemu = s.spawn(|| {
			let status = run_qemu(
				cmd,
				debugger,
				qemu,
				temp_dir,
				libs2e,
				libs2e_dir,
				s2e_config,
				max_processes,
				image,
				serial_out,
				qmp_socket.as_deref(),
			);
			if status.success() {
				Ok(())
			} else {
				tracing::error!(?status, "qemu exited with error code");
				Err(())
			}
		});
		if let Some(qmp_socket) = &qmp_socket {
			tracing::debug!(?qmp_socket, "Starting QMP server over");
			run_qmp(qmp_socket);
		}
		let ret = qemu.join().unwrap();
		match UnixStream::connect(ipc_socket) {
			Ok(conn) => conn.shutdown(Shutdown::Both).unwrap(),
			Err(_) => {}
		}
		ipc.join().unwrap();
		ret
	});
	cmd.remove(ipc_socket);
	res
}

fn run_qemu(
	cmd: &mut Cmd,
	debugger: bool,
	qemu: &Path,
	temp_dir: &Path,
	libs2e: &Path,
	libs2e_dir: &Path,
	s2e_config: &Path,
	max_processes: u16,
	image: &Path,
	serial_out: &Path,
	qmp_socket: Option<&Path>,
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

	if debugger {
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
	if let Some(qmp_socket) = qmp_socket {
		command.arg("-qmp").arg({
			let mut line = OsString::new();
			line.push("unix:");
			line.push(qmp_socket);
			line.push(",server,nowait");
			line
		});
	}
	command
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
	cmd.command_spawn_wait(&mut command)
}

fn run_qmp(socket: &Path) {
	let stream = 'outer: {
		let mut attempt = 0;
		let result = loop {
			attempt += 1;
			match UnixStream::connect(socket) {
				Ok(stream) => break 'outer stream,
				Err(err) if attempt > 10 => break err,
				Err(_) => {}
			}
			thread::sleep(Duration::from_millis(10));
		};
		tracing::error!(?result, ?socket, "failed to connect to socket");
		return;
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
			Ok(response) => tracing::info!(?response, "QMP"),
			Err(QmpError::EndOfFile) => return,
			Err(err) => unreachable!("{:?}", err),
		}
	}
}

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
