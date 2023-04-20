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
	time::{Duration, Instant},
};

use eframe::egui::Context;
use graphui::{EmbeddingParameters, Graph2D};
use ipc::{GraphIpc, IpcError, IpcMessage};
use qmp_client::{QmpClient, QmpCommand, QmpError, QmpEvent};

use crate::{cmd::Cmd, gui::Model, run::session::S2EConfig, SessionConfig};

mod session;
mod run;
mod control;
