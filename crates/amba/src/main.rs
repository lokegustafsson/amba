use std::{
	env, fs,
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
/// Set `AMBA_SRC_DIR` to a directory containing the amba source tree.
/// Set `AMBA_DATA_DIR` to a directory where amba should read and write
/// intermediate artifacts such as disk images.
#[derive(clap::Parser, Debug)]
#[command(about, verbatim_doc_comment)]
enum Args {
	Init(InitArgs),
	Run(RunArgs),
}
#[derive(clap::Args, Debug)]
/// Initialize `$AMBA_DATA_DIR`
struct InitArgs {}
/// Run QEMU+S2E+libamba
#[derive(clap::Args, Debug)]
struct RunArgs {
	host_path_to_executable: PathBuf,
}

fn main() -> ExitCode {
	tracing::subscriber::set_global_default(
		tracing_subscriber::FmtSubscriber::builder()
			.with_max_level(tracing::Level::TRACE)
			.with_timer(UptimeHourMinuteSeconds::default())
			.finish(),
	)
	.expect("enabling global logger");

	let args: Args = clap::Parser::parse();
	let src_dir = match env::var_os("AMBA_SRC_DIR") {
		Some(dir) => PathBuf::from(dir),
		None => fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/../../")).unwrap(),
	};
	let data_dir = match env::var_os("AMBA_DATA_DIR") {
		Some(dir) => PathBuf::from(dir),
		None => home::home_dir().unwrap().join("amba"),
	};
	let dependencies_dir = Path::new(env!("AMBA_DEPENDENCIES_DIR"));

	tracing::info!(AMBA_SRC_DIR = ?src_dir);
	tracing::info!(AMBA_DATA_DIR = ?data_dir);
	tracing::info!(AMBA_DEPENDENCIES_DIR = ?dependencies_dir);
	tracing::info!(?args);

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
