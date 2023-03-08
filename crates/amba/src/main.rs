use std::{env, path::PathBuf, process::ExitCode, time::Instant};

use tracing_subscriber::fmt;

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
pub struct InitArgs {
	#[arg(short, long)]
	force: bool,
	/// Download guest images built by the S2E team from Google Drive rather than
	/// building them locally
	#[arg(short, long)]
	download: bool,
}

/// Run QEMU+S2E+libamba
#[derive(clap::Args, Debug)]
pub struct RunArgs {
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

	let data_dir = &match env::var_os("AMBA_DATA_DIR") {
		Some(dir) => PathBuf::from(dir),
		None => home::home_dir().unwrap().join("amba"),
	};

	tracing::info!(debug_assertions = cfg!(debug_assertions));
	tracing::info!(AMBA_DEPENDENCIES_DIR);
	tracing::info!(AMBA_SRC_DIR);
	tracing::info!(AMBA_DATA_DIR = ?data_dir);
	tracing::info!(?args);

	let cmd = &mut cmd::Cmd::get();
	let res = match args {
		Args::Init(args) => init::init(cmd, data_dir, args),
		Args::Run(args) => run::run(cmd, data_dir, args),
	};
	match res {
		Ok(()) => ExitCode::SUCCESS,
		Err(()) => ExitCode::FAILURE,
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
