use std::{
	env,
	path::{Path, PathBuf},
	process::ExitCode,
	time::Instant,
};

use tracing_subscriber::fmt;

use crate::run::SessionConfig;

mod cmd;
mod init;
mod run;

/// The executable component of amba that runs QEMU+S2E+libamba as a subprocess
///
/// Set `AMBA_DATA_DIR` to a directory where amba should read and write
/// run time artifacts such as disk images. The default is `$HOME/amba`.
#[derive(clap::Parser, Debug)]
#[command(about, verbatim_doc_comment)]
enum Args {
	Init(InitArgs),
	Run(RunArgs),
}

/// Initialize `$AMBA_DATA_DIR`
#[derive(clap::Args, Debug)]
struct InitArgs {}

/// Run QEMU+S2E+libamba
#[derive(clap::Args, Debug)]
struct RunArgs {
	host_path_to_executable: PathBuf,
}

const AMBA_DEPENDENCIES_DIR: &str = env!("AMBA_DEPENDENCIES_DIR");
const AMBA_SRC_DIR: &str = env!("AMBA_SRC_DIR");

fn main() -> ExitCode {
	tracing::subscriber::set_global_default(
		tracing_subscriber::FmtSubscriber::builder()
			.with_max_level(tracing::Level::TRACE)
			.with_timer(UptimeHourMinuteSeconds::default())
			.finish(),
	)
	.expect("enabling global logger");

	let args: Args = clap::Parser::parse();

	let data_dir = match env::var_os("AMBA_DATA_DIR") {
		Some(dir) => PathBuf::from(dir),
		None => home::home_dir().unwrap().join("amba"),
	};

	tracing::info!(AMBA_DEPENDENCIES_DIR);
	tracing::info!(AMBA_SRC_DIR);
	tracing::info!(AMBA_DATA_DIR = ?data_dir);
	tracing::info!(?args);

	let dependencies_dir = Path::new(AMBA_DEPENDENCIES_DIR);
	let src_dir = Path::new(AMBA_SRC_DIR);

	let cmd = &mut cmd::Cmd::get();
	match args {
		Args::Init(InitArgs {}) => init::init(cmd, &src_dir, &data_dir),
		Args::Run(RunArgs {
			host_path_to_executable,
		}) => run::run(
			cmd,
			&data_dir,
			&dependencies_dir,
			&SessionConfig {
				path_to_executable: host_path_to_executable,
			},
		),
	}
}

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
