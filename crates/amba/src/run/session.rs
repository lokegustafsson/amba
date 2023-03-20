//! Populating the session directory

use std::{
	error::Error,
	path::{Path, PathBuf},
};

use include_dir::{include_dir, Dir};
use recipe::{FileSource, Recipe};
use serde::Serialize;
use tera::{Context, Tera};

use crate::cmd::Cmd;

/// All data required to populate the templates in `crates/amba/templates/`.
/// The templates are kept as close to the upstream S2E templates as possible.
#[derive(Serialize)]
pub struct S2EConfig {
	creation_time: &'static str,
	project_dir: PathBuf,
	host_files_dir: PathBuf,
	use_seeds: bool,
	project_name: &'static str,
	modules: Vec<[String; 1]>,
	processes: Vec<String>,
	use_cupa: bool,
	target_lua_template: &'static str,
	custom_lua_string: &'static str,
	project_type: &'static str,
	image_arch: &'static str,
	target_bootstrap_template: &'static str,
	target: Target,
	dynamically_linked: bool,
	sym_args: Vec<u8>,
	enable_pov_generation: bool,
	enable_tickler: bool,
	has_guestfs: bool,
	guestfs_paths: Vec<PathBuf>,
	use_test_case_generator: bool,
	enable_cfi: bool,
}

#[derive(Serialize)]
pub struct Target {
	arch: &'static str,
	name: String,
	names: Vec<String>,
	args: Args,
}

#[derive(Serialize)]
pub struct Args {
	symbolic_file_names: Vec<String>,
	resolved_args: Vec<String>,
}

const LIBRARY_LUA: &str = include_str!("../../data/library.lua");
const TEMPLATE_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/templates");
const BOOTSTRAP_SH_CONTENT: &str = "
./s2ecmd get bootstrap.elf
chmod +x ./bootstrap.elf
RUST_BACKTRACE=1 ./bootstrap.elf 2>&1
";

const CUSTOM_LUA_STRING: &str = r#"
add_plugin("AmbaPlugin")
"#;

impl S2EConfig {
	/// Default template parameters. Update this to change the S2E run time
	/// configuration.
	pub fn new(cmd: &mut Cmd, session_dir: &Path, recipe_path: &Path, recipe: &Recipe) -> Self {
		let host_files_dir = session_dir.join("hostfiles");
		cmd.create_dir_all(&host_files_dir);
		for (guest_path, source) in &recipe.files {
			if let FileSource::Host(host_path) | FileSource::SymbolicHost { host_path, .. } = source
			{
				let guest_path = Path::new(guest_path);
				assert!(guest_path.is_relative());
				cmd.copy(
					recipe_path.parent().unwrap().join(host_path),
					host_files_dir.join(guest_path),
				);
			}
		}
		cmd.write(
			host_files_dir.join("recipe.json"),
			&serde_json::to_vec(recipe).unwrap(),
		);

		Self {
			creation_time: "CREATION_TIME",
			project_dir: session_dir.to_owned(),
			host_files_dir,
			use_seeds: false,
			project_name: "PROJECT_NAME",
			modules: vec![[recipe.executable_path.clone()]],
			processes: vec![recipe.executable_path.clone()],
			use_cupa: true,
			target_lua_template: "s2e-config.linux.lua",
			custom_lua_string: CUSTOM_LUA_STRING,
			project_type: "linux",
			image_arch: "x86_64",
			target_bootstrap_template: "bootstrap.linux.sh",
			target: Target {
				arch: "x86_64",
				name: recipe.executable_path.clone(),
				names: vec![recipe.executable_path.clone()],
				args: Args {
					symbolic_file_names: vec![],
					resolved_args: vec![],
				},
			},
			dynamically_linked: true,
			sym_args: vec![],
			enable_pov_generation: false,
			enable_tickler: false,
			has_guestfs: false,
			guestfs_paths: Vec::new(),
			use_test_case_generator: true,
			enable_cfi: false,
		}
	}

	pub fn save_to(&self, cmd: &mut Cmd, dependencies_dir: &Path, session_dir: &Path) {
		assert!(session_dir.exists());
		cmd.write(session_dir.join("library.lua"), LIBRARY_LUA);
		cmd.symlink(
			dependencies_dir.join("bin/guest-tools64"),
			session_dir.join("guest-tools64"),
		);
		cmd.symlink(
			dependencies_dir.join("bin/guest-tools32"),
			session_dir.join("guest-tools32"),
		);
		cmd.symlink(
			dependencies_dir.join("bin/bootstrap"),
			self.host_files_dir.join("bootstrap.elf"),
		);
		cmd.write(
			self.host_files_dir.join("bootstrap.sh"),
			BOOTSTRAP_SH_CONTENT,
		);

		tracing::debug!(TEMPLATE_DIR = ?TEMPLATE_DIR.path(), "Using templates from");
		let mut renderer = Renderer::new(cmd, session_dir, self);
		renderer.render("s2e-config.lua");
	}
}

struct Renderer<'a> {
	cmd: &'a mut Cmd,
	session_dir: &'a Path,
	tera: Tera,
	context: Context,
}

impl<'a> Renderer<'a> {
	fn new(cmd: &'a mut Cmd, session_dir: &'a Path, config: &S2EConfig) -> Self {
		Self {
			cmd,
			session_dir,
			tera: {
				let mut tera = Tera::default();
				tera.add_raw_templates(TEMPLATE_DIR.entries().iter().map(|entry| {
					let file = entry.as_file().unwrap();
					(
						file.path().file_name().unwrap().to_str().unwrap(),
						std::str::from_utf8(file.contents()).unwrap(),
					)
				}))
				.unwrap_or_else(|err| Renderer::handle_error("template_dir", err));
				tera
			},
			context: Context::from_serialize(config).unwrap(),
		}
	}

	fn render(&mut self, name: &'static str) {
		match self.tera.render(name, &self.context) {
			Ok(content) => self.cmd.write(self.session_dir.join(name), content),
			Err(err) => Self::handle_error(name, err),
		}
	}

	/// Inspect the `tera::Error` to possibly pretty-print it before panicking.
	fn handle_error(name: &'static str, err: tera::Error) -> ! {
		let msg = match (&err.kind, err.source()) {
			(tera::ErrorKind::Msg(msg), None) => Some(msg.clone()),
			(tera::ErrorKind::Msg(msg), Some(inner)) => {
				match (
					inner.downcast_ref::<tera::Error>().map(|e| &e.kind),
					inner.source(),
				) {
					(Some(tera::ErrorKind::Msg(inner_msg)), None) => {
						Some(format!("{msg}\n{inner_msg}"))
					}
					_ => None,
				}
			}
			_ => None,
		};
		if let Some(msg) = msg {
			tracing::error!("Error within {name}:\n{msg}");
		}
		panic!("Tera::render failed at {err:#?}");
	}
}
