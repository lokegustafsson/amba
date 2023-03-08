use std::{path::Path, process::Command};

use crate::{cmd::Cmd, InitArgs, AMBA_SRC_DIR};

mod build;
mod download;

pub fn init(
	cmd: &mut Cmd,
	data_dir: &Path,
	InitArgs { force, download }: InitArgs,
) -> Result<(), ()> {
	let build_guest_images_flake_ref = &format!("path:{AMBA_SRC_DIR}#build-guest-images",);
	let builder_version = cmd
		.command_capture_stdout(Command::new("nix").args([
			"build",
			build_guest_images_flake_ref,
			"--no-link",
			"--print-out-paths",
		]))
		.unwrap();
	let version_file = &data_dir.join("version.txt");
	let existing_builder_version = version_file.exists().then(|| cmd.read(version_file));

	if !force && existing_builder_version.as_ref() == Some(&builder_version) {
		tracing::info!("guest images already up to date; force rebuild with --force");
		return Ok(());
	}
	version_file.exists().then(|| cmd.remove(version_file));
	if download {
		tracing::info!("downloading guest images");
		download::force_init_download(cmd, data_dir)?;
	} else {
		tracing::info!("building guest images");
		build::force_init_build(cmd, data_dir, build_guest_images_flake_ref)?;
	}
	assert!(data_dir
		.join("images/ubuntu-22.04-x86_64/image.json")
		.exists());
	assert!(data_dir
		.join("images/ubuntu-22.04-x86_64/image.raw.s2e")
		.exists());
	assert!(data_dir
		.join("images/ubuntu-22.04-x86_64/image.raw.s2e.ready")
		.exists());

	cmd.write(version_file, builder_version);
	Ok(())
}
