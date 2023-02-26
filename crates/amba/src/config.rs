use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use serde::Serialize;

use crate::cmd::Cmd;

#[derive(Serialize)]
pub struct S2EConfig {
	enable_tickler: bool,
	project_dir: PathBuf,
	use_seeds: bool,
	seeds_dir: Option<PathBuf>,
	has_guestfs: bool,
	guestfs_paths: Vec<PathBuf>,
	modules: Vec<[String; 1]>,
	processes: Vec<String>,
	use_cupa: bool,
	enable_pov_generation: bool,
	use_test_case_generator: bool,
	recipes_dir: Option<PathBuf>,
	enable_cfi: bool,
	custom_lua_string: String,
}

const TEMPLATE: &str = include_str!("lua/s2e-config.template.lua");
const LIBRARY: &str = include_str!("lua/library.lua");

impl S2EConfig {
	pub fn new(project_dir: PathBuf, tracked_process_file_names: &[&str]) -> Self {
		Self {
			enable_tickler: false,
			project_dir,
			use_seeds: false,
			seeds_dir: None,
			has_guestfs: false,
			guestfs_paths: Vec::new(),
			modules: tracked_process_file_names
				.iter()
				.map(|&name| [name.to_owned()])
				.collect(),
			processes: tracked_process_file_names
				.iter()
				.map(|&name| name.to_owned())
				.collect(),
			use_cupa: true,
			enable_pov_generation: false,
			use_test_case_generator: true,
			recipes_dir: None,
			enable_cfi: true,
			custom_lua_string: "".to_string(),
		}
	}

	pub fn save_to(&self, cmd: &mut Cmd, path: &Path) {
		let parent = path.parent().unwrap();
		assert!(parent.exists());
		cmd.write(
			path,
			Handlebars::new().render_template(TEMPLATE, &self).unwrap(),
		);
		cmd.write(parent.join("library.lua"), LIBRARY);
	}
}
