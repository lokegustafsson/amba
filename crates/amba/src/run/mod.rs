//! The run subcommand

use std::{
	ffi::{OsStr, OsString},
	path::Path,
	process::{Command, ExitStatus},
};

use chrono::offset::Local;

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
	RunArgs {
		host_path_to_executable,
		debugger,
	}: RunArgs,
) -> Result<(), ()> {
	if !data_dir_has_been_initialized(cmd, data_dir) {
		tracing::error!(
			?data_dir,
			"AMBA_DATA_DIR has not been initialized"
		);
		return Err(());
	}

	let session_dir = &data_dir.join(Local::now().format("%Y-%m-%dT%H:%M:%S").to_string());
	if session_dir.exists() {
		tracing::error!(
			?session_dir,
			"session_dir already exists, are multiple amba instances running concurrently?"
		);
		return Err(());
	}
	cmd.create_dir_all(session_dir);
	// Populate the `session_dir`
	{
		let executable_name = host_path_to_executable.file_name().unwrap();
		cmd.copy(
			&host_path_to_executable,
			session_dir.join(executable_name),
		);
		S2EConfig::new(session_dir, executable_name).save_to(cmd, dependencies_dir, session_dir);
	}

	// supporting single- vs multi-path
	let s2e_mode = match true {
		true => "s2e",
		false => "s2e_sp",
	};
	let arch = "x86_64";

	let gdb = &dependencies_dir.join("bin/gdb");
	let qemu = &dependencies_dir.join(format!("bin/qemu-system-{arch}"));
	let libs2e_dir = &dependencies_dir.join("share/libs2e");
	let libs2e = &libs2e_dir.join(format!("libs2e-{arch}-{s2e_mode}.so"));
	let s2e_config = &session_dir.join("s2e-config.lua");
	let max_processes = 1;
	let image = &data_dir.join("images/ubuntu-22.04-x86_64/image.raw.s2e");
	let serial_out = &session_dir.join("serial.txt");

	let status = run_qemu(
		cmd,
		debugger,
		gdb,
		qemu,
		session_dir,
		libs2e,
		libs2e_dir,
		s2e_config,
		max_processes,
		image,
		serial_out,
	);
	if status.success() {
		Ok(())
	} else {
		tracing::error!(?status, "qemu exited with error code");
		Err(())
	}
}
fn run_qemu(
	cmd: &mut Cmd,
	debugger: bool,
	gdb: &Path,
	qemu: &Path,
	session_dir: &Path,
	libs2e: &Path,
	libs2e_dir: &Path,
	s2e_config: &Path,
	max_processes: u16,
	image: &Path,
	serial_out: &Path,
) -> ExitStatus {
	assert!(gdb.exists());
	assert!(qemu.exists());
	assert!(libs2e.exists());
	assert!(s2e_config.exists());
	assert!(libs2e_dir.exists());

	let mut command = if debugger {
		// Set env vars within GDB, run QEMU in GDB
		cmd.write(session_dir.join("gdb.ini"), {
			use std::fmt::Write;
			let mut init = String::new();
			writeln!(init, "handle SIGUSR1 noprint").unwrap();
			writeln!(init, "handle SIGUSR2 noprint").unwrap();
			writeln!(init, "set disassembly-flavor intel").unwrap();
			writeln!(init, "set print pretty on").unwrap();
			for (k, v) in [
				(
					"LD_PRELOAD",
					crate::util::os_str_to_escaped_ascii(libs2e),
				),
				(
					"S2E_CONFIG",
					crate::util::os_str_to_escaped_ascii(s2e_config),
				),
				(
					"S2E_SHARED_DIR",
					crate::util::os_str_to_escaped_ascii(libs2e_dir),
				),
				("S2E_MAX_PROCESSES", max_processes.to_string()),
				("S2E_UNBUFFERED_STREAM", "1".to_owned()),
			] {
				writeln!(init, "set environment {k}={v}").unwrap();
			}
			init
		});
		let mut command = Command::new(gdb);
		command.args(["--init-command=gdb.ini", "--args"]).arg(qemu);
		command
	} else {
		// Set env vars directly, run QEMU
		let mut command = Command::new(qemu);
		command
			.env("LD_PRELOAD", libs2e)
			.env("S2E_CONFIG", s2e_config)
			.env("S2E_SHARED_DIR", libs2e_dir)
			.env("S2E_MAX_PROCESSES", max_processes.to_string())
			.env("S2E_UNBUFFERED_STREAM", "1");
		command
	};
	command.current_dir(session_dir);

	// QEMU arguments
	if max_processes > 1 {
		command.arg("-nographic");
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
		])
		.arg({
			let mut line = OsString::new();
			line.push("file:");
			line.push(serial_out);
			line
		})
		.args([
			"-net",
			"none",
			"-net",
			"nic,model=e1000",
			"-loadvm",
			"ready",
		]);
	// Run subprocess
	cmd.command_spawn_wait(&mut command)
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
