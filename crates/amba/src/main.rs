use std::{env, path::PathBuf, process::ExitCode, time::Instant};

use chrono::offset::Local;
use rand::{distributions::Alphanumeric, Rng};
use tracing_subscriber::{filter::targets::Targets, fmt, layer::Layer};

mod cmd;
mod init;
mod run;

/// The executable component of amba that runs QEMU+S2E+libamba as a subprocess
///
/// Set `AMBA_DATA_DIR` to a directory where amba should read and write
/// run time artifacts such as disk images. The default is `$XDG_DATA_HOME/amba`
/// or `$HOME/.local/share/amba`.
#[derive(clap::Parser, Debug)]
#[command(about, verbatim_doc_comment)]
enum Args {
	Init(InitArgs),
	Run(RunArgs),
	Gui(GuiArgs),
}

/// Initialize `$AMBA_DATA_DIR`
#[derive(clap::Args, Debug)]
pub struct InitArgs {
	/// Perform initialization even if amba has already been initialized.
	#[arg(short, long)]
	force: bool,
	/// Download guest images built by the S2E team from Google Drive rather than
	/// building them locally.
	#[arg(short, long)]
	download: bool,
}

/// Run QEMU+S2E+libamba
#[derive(clap::Args, Debug)]
pub struct RunArgs {
	/// Path to a recipe file specifying the run
	recipe_path: PathBuf,
	/// Start QEMU in a paused state, to attach a debugger or profiler
	#[arg(short, long)]
	debugger: bool,
	/// Connect to QEMU:s QMP server
	#[arg(short, long)]
	qmp: bool,
}

#[derive(clap::Args, Debug)]
pub struct GuiArgs {}

/// The nix store path of the script that builds guest images.
const AMBA_BUILD_GUEST_IMAGES_SCRIPT: &str = env!("AMBA_BUILD_GUEST_IMAGES_SCRIPT");

fn main() -> ExitCode {
	tracing::subscriber::set_global_default(
		Targets::new()
			.with_target("h2", tracing::Level::INFO)
			.with_target(
				"tokio_util::codec::framed_impl",
				tracing::Level::DEBUG,
			)
			.with_default(tracing::Level::TRACE)
			.with_subscriber(
				tracing_subscriber::FmtSubscriber::builder()
					.with_max_level(tracing::Level::TRACE)
					.with_timer(UptimeHourMinuteSeconds::default())
					.finish(),
			),
	)
	.expect("enabling global logger");

	let args: Args = clap::Parser::parse();

	let data_dir = &match env::var_os("AMBA_DATA_DIR") {
		Some(dir) => PathBuf::from(dir),
		None => dirs::data_dir().unwrap().join("amba"),
	};
	let dependencies_dir = &match env::var_os("RUN_TIME_AMBA_DEPENDENCIES_DIR") {
		Some(dir) => PathBuf::from(dir),
		None => PathBuf::from(env!("COMPILE_TIME_AMBA_DEPENDENCIES_DIR")),
	};

	tracing::info!(debug_assertions = cfg!(debug_assertions));
	tracing::info!(AMBA_DEPENDENCIES_DIR = ?dependencies_dir);
	tracing::info!(AMBA_BUILD_GUEST_IMAGES_SCRIPT);
	tracing::info!(AMBA_DATA_DIR = ?data_dir);
	tracing::info!(?args);

	let cmd = &mut cmd::Cmd::get();
	let res = match args {
		Args::Init(args) => init::init(cmd, data_dir, args),
		Args::Run(args) => {
			let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S");
			let mut rng = rand::thread_rng();
			let random: String = (0..6).map(|_| rng.sample(Alphanumeric) as char).collect();

			let session_dir = &data_dir.join(timestamp.to_string());
			let temp_dir = &env::temp_dir().join(format!("amba-{timestamp}-{random}"));
			run::run(
				cmd,
				dependencies_dir,
				data_dir,
				session_dir,
				temp_dir,
				args,
			)
		}
		Args::Gui(_) => Ok(amba_gui::main()),
	};
	match res {
		Ok(()) => ExitCode::SUCCESS,
		Err(()) => ExitCode::FAILURE,
	}
}

/// A timer to add `{h}h{m}m{s}s` to logs.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UptimeHourMinuteSeconds {
	epoch: Instant,
}

impl Default for UptimeHourMinuteSeconds {
	fn default() -> Self {
		UptimeHourMinuteSeconds {
			epoch: Instant::now(),
		}
	}
}

impl fmt::time::FormatTime for UptimeHourMinuteSeconds {
	fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
		let seconds = self.epoch.elapsed().as_secs();
		let h = seconds / (60 * 60);
		let m = (seconds / 60) % (60 * 60);
		let s = seconds % 60;
		write!(w, "{h}h{m}m{s}s")
	}
}
