//! This executable will run on startup within the guest, and is responsible
//! for starting the analyzed executable with a correctly made-symbolic
//! environment. It replaces the S2E bootstrap.sh script.

#[cfg(all(
	not(test),
	any(
		not(target_arch = "x86_64"),
		not(target_vendor = "unknown"),
		not(target_os = "linux"),
		not(target_env = "musl"),
	)
))]
compile_error!("bootstrap supports only 'x86_64-unknown-linux-musl'",);

use std::{
	fs::{self, File},
	io,
	os::unix::{fs::PermissionsExt, process::CommandExt},
	path::Path,
	process::{Command, Stdio},
};

use nix::unistd;
use recipe::{FileSource, Recipe};

const RECIPE_PATH: &str = "./recipe.json";

fn main() {
	println!("bootstrap (rust musl binary) started");

	let recipe = Recipe::deserialize_from(fs::read_to_string(RECIPE_PATH).unwrap().as_bytes())
		.expect("deserializing Recipe");
	if true {
		todo!("recipe loaded, but the bootstrap executable is a work-in-progress");
	}

	assert!(Path::new("./s2ecmd").exists());
	assert_eq!(unistd::gethostname().unwrap(), "s2e??");
	assert!(run_capture(&["mount"]).contains("/tmp type tmpfs"));

	run(&["sudo", "sysctl", "-w", "debug.exception-trace=0"]);
	run(&["ulimit", "-c", "0"]);
	run(&["sudo", "swapoff", "-a"]);
	run(&["sudo", "modprobe", "s2e"]);

	for (guest_path, source) in &recipe.files {
		run(&["./s2ecmd", "get", guest_path]);
		let guest_path = Path::new(guest_path);
		assert!(guest_path.is_relative());
		match source {
			FileSource::Host(_) => {}
			FileSource::SymbolicContent { symbolic, .. }
			| FileSource::SymbolicHost { symbolic, .. } => {
				let tmp_guest_path = &Path::new("/tmp").join(guest_path);
				fs::rename(guest_path, tmp_guest_path).unwrap();

				let total_len = fs::metadata(tmp_guest_path).unwrap().len();
				let symbolic: String = symbolic
					.iter()
					.map(|range| {
						format!(
							"{}-{}",
							range.start(),
							range.len().unwrap_or(total_len - range.start())
						)
					})
					.collect::<Vec<String>>()
					.join(" ");
				Command::new("./s2ecmd")
					.env("S2E_SYMFILE_RANGES", &symbolic)
					.args(["symbfile", "1"])
					.arg(tmp_guest_path)
					.spawn()
					.unwrap()
					.wait()
					.unwrap();
			}
		};
	}

	fs::metadata(&recipe.executable_path)
		.unwrap()
		.permissions()
		.set_mode(0o555);
	let mut child = Command::new(&recipe.executable_path)
		.arg0(recipe.arg0.unwrap_or(recipe.executable_path))
		.stdin(Stdio::piped())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.spawn()
		.unwrap();

	io::copy(
		&mut File::open(recipe.stdin_path).unwrap(),
		&mut child.stdin.as_mut().unwrap(),
	)
	.unwrap();

	let status = child.wait().unwrap();
	println!("analyzed program status: {status:?}");
}

fn run(cmd: &[&str]) {
	Command::new(cmd[0])
		.args(&cmd[1..])
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
}
fn run_capture(cmd: &[&str]) -> String {
	let output = Command::new(cmd[0]).args(&cmd[1..]).output().unwrap();
	assert!(output.status.success());
	String::from_utf8(output.stdout).unwrap()
}