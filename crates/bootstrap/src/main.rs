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
	fs::{self, File, Permissions},
	io,
	os::unix::{
		fs::{MetadataExt, PermissionsExt},
		process::CommandExt,
	},
	path::Path,
	process::{Command, Stdio},
	thread,
	time::Duration,
};

use recipe::{ArgumentSource, EnvVarSource, FileSource, Recipe, SymbolicRange};
use tracing_subscriber::{filter::targets::Targets, layer::Layer};

const RECIPE_PATH: &str = "recipe.json";

fn main() {
	tracing::subscriber::set_global_default(
		Targets::new()
			//.with_target("h2", tracing::Level::INFO)
			.with_default(tracing::Level::TRACE)
			.with_subscriber(
				tracing_subscriber::FmtSubscriber::builder()
					.with_max_level(tracing::Level::TRACE)
					.without_time()
					.finish(),
			),
	)
	.expect("enabling global logger");

	tracing::info!("started within guest");

	assert!(Path::new("./s2ecmd").exists());
	assert_eq!(nix::unistd::gethostname().unwrap(), "s2e");

	if !run_capture(&["mount"]).contains("/tmp type tmpfs") {
		run(&[
			"sudo",
			"mount",
			"-t",
			"tmpfs",
			"-osize=10m",
			"tmpfs",
			"/tmp",
		]);
		let mount_output = run_capture(&["mount"]);
		assert!(
			mount_output.contains("/tmp type tmpfs"),
			"expected /tmp on tmpfs in mount output:\n{mount_output}"
		);
	}

	nix::sys::resource::setrlimit(nix::sys::resource::Resource::RLIMIT_CORE, 0, 0).unwrap();
	run(&["sudo", "sysctl", "-w", "debug.exception-trace=0"]);
	run(&["sudo", "swapoff", "-a"]);
	run(&["sudo", "modprobe", "s2e"]);
	run(&["./s2ecmd", "get", RECIPE_PATH]);

	let recipe = Recipe::deserialize_from(fs::read_to_string(RECIPE_PATH).unwrap().as_bytes())
		.expect("deserializing Recipe");

	for (guest_path, source) in &recipe.files {
		run(&["./s2ecmd", "get", guest_path]);
		let guest_path = Path::new(guest_path);
		assert!(guest_path.is_relative());
		match source {
			FileSource::Host(_) => {}
			FileSource::SymbolicContent { symbolic, .. }
			| FileSource::SymbolicHost { symbolic, .. } => {
				let tmp_guest_path = &Path::new("/tmp").join(guest_path);
				fs::copy(guest_path, tmp_guest_path).unwrap();
				symbfile(tmp_guest_path, symbolic);
			}
		};
	}

	if fs::metadata(&recipe.executable_path).unwrap().uid() == nix::unistd::Uid::current().as_raw()
	{
		fs::set_permissions(
			&recipe.executable_path,
			Permissions::from_mode(0o555),
		)
		.unwrap();
	}
	tracing::info!(
		recipe.executable_path,
		"running executable to analyze"
	);

	// Sleeping a little allows the system to become idle and the kernel to do some
	// bookkeeping before we launch the executable, leading to state forks. This
	// causes the state to fork with less work to do, which beneficial for
	// performance.
	thread::sleep(Duration::from_millis(50));

	let mut child = {
		let mut cmd = Command::new(&recipe.executable_path);
		cmd.arg0(recipe.arg0.unwrap_or(recipe.executable_path))
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.stderr(Stdio::null());
		for arg in recipe.arguments {
			match arg {
				ArgumentSource::Concrete(value) => {
					cmd.arg(value);
				}
				ArgumentSource::Symbolic { seed, .. } => todo!("symbolic argv (arg = `{seed:?}`)"),
			}
		}
		for env_key_to_remove in &recipe.environment.remove {
			cmd.env_remove(env_key_to_remove);
		}
		if !recipe.environment.inherit {
			cmd.env_clear();
		}
		for (env_key, env_value) in &recipe.environment.add {
			match env_value {
				EnvVarSource::Concrete(value) => {
					cmd.env(env_key, value);
				}
				EnvVarSource::Symbolic { value, .. } => {
					todo!("symbolic envp ({env_key:?}={value:?})");
				}
			}
		}

		cmd.spawn().unwrap()
	};

	io::copy(
		&mut File::open(recipe.stdin_path).unwrap(),
		&mut child.stdin.as_mut().unwrap(),
	)
	.unwrap();

	let status = child.wait().unwrap();
	tracing::info!("analyzed program status: {status:?}");
}

fn run(cmd: &[&str]) {
	tracing::trace!(?cmd, "running");
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

fn symbfile(path: &Path, symbolic: &[SymbolicRange]) {
	let total_len = fs::metadata(path).unwrap().len();
	let symbolic: String = symbolic
		.iter()
		.filter(|range| range.start() < total_len)
		.map(|range| {
			format!(
				"{}-{}",
				range.start(),
				range.len().min(total_len - range.start())
			)
		})
		.collect::<Vec<String>>()
		.join(" ");
	tracing::trace!(?path, ?symbolic, "Running ./s2ecmd symfile with");
	Command::new("./s2ecmd")
		.env("S2E_SYMFILE_RANGES", &symbolic)
		.args(["symbfile", "1"])
		.arg(path)
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
}
