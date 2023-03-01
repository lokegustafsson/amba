use std::path::{Path, PathBuf};

use serde::Serialize;
use tera::{Context, Tera};

use crate::cmd::Cmd;

#[derive(Serialize)]
pub struct S2EConfig {
	// Common
	enable_pov_generation: bool,
	use_seeds: bool,
	enable_tickler: bool,
	// For `s2e-config.lua`
	project_dir: PathBuf,
	seeds_dir: Option<PathBuf>,
	allow_s2eput_to_host: bool,
	has_guestfs: bool,
	guestfs_paths: Vec<PathBuf>,
	modules: Vec<[String; 1]>,
	processes: Vec<String>,
	use_cupa: bool,
	use_test_case_generator: bool,
	recipes_dir: Option<PathBuf>,
	enable_cfi: bool,
	custom_lua_string: String,
	// For `bootstrap.sh`
	project_type: &'static str,
	image_arch: &'static str,
	target: Target,
}
#[derive(Serialize)]
pub struct Target {
	name: String,
	file_names_to_s2eget: Vec<String>,
	args_space_concatenated_symbolic_file_names: String,
	args_space_concatenated_all: String,
}

const TEMPLATE: &str = include_str!("../../template/template.s2e-config.lua");
const LIBRARY: &str = include_str!("../../template/library.lua");
const BOOTSTRAP: &str = include_str!("../../template/template.bootstrap.sh");

impl S2EConfig {
	pub fn new(session_dir: &Path, executable_file_name: &str) -> Self {
		Self {
			enable_tickler: false,
			project_dir: session_dir.to_owned(),
			use_seeds: false,
			seeds_dir: None,
			allow_s2eput_to_host: false,
			has_guestfs: false,
			guestfs_paths: Vec::new(),
			modules: vec![[executable_file_name.to_owned()]],
			processes: vec![executable_file_name.to_owned()],
			use_cupa: true,
			enable_pov_generation: false,
			use_test_case_generator: true,
			recipes_dir: None,
			enable_cfi: true,
			custom_lua_string: "".to_string(),
			project_type: "linux",
			image_arch: "x86_64",
			target: Target {
				name: executable_file_name.to_owned(),
				file_names_to_s2eget: vec![executable_file_name.to_owned()],
				args_space_concatenated_symbolic_file_names: "".to_owned(),
				args_space_concatenated_all: "".to_owned(),
			},
		}
	}

	pub fn save_to(&self, cmd: &mut Cmd, session_dir: &Path) {
		assert!(session_dir.exists());
		let mut tera = Tera::default();
		let context = Context::from_serialize(self).unwrap();
		cmd.write(
			session_dir.join("s2e-config.lua"),
			tera.render_str(TEMPLATE, &context).unwrap(),
		);
		cmd.write(session_dir.join("library.lua"), LIBRARY);
		cmd.write(
			session_dir.join("bootstrap.sh"),
			tera.render_str(BOOTSTRAP, &context).unwrap(),
		);
	}
}
