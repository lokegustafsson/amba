use std::{path::Path, process::Command};

use crate::{cmd::Cmd, InitArgs, AMBA_SRC_DIR};

pub fn init(cmd: &mut Cmd, data_dir: &Path, InitArgs { force }: InitArgs) -> Result<(), ()> {
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
	tracing::info!("building guest images");
	force_init(cmd, data_dir, build_guest_images_flake_ref)?;
	cmd.write(version_file, builder_version);
	Ok(())
}
pub fn force_init(
	cmd: &mut Cmd,
	data_dir: &Path,
	build_guest_images_flake_ref: &str,
) -> Result<(), ()> {
	let images = &data_dir.join("images");
	let images_build = &data_dir.join("images-build");
	if images.exists() {
		remove_images(cmd, images);
	}
	if images_build.exists() {
		remove_images_build(cmd, images_build);
	}
	let build_result = cmd.command_spawn_wait(
		Command::new("nix")
			.args(["run", build_guest_images_flake_ref, "--"])
			.args([images_build, images]),
	);
	if !build_result.success() {
		tracing::error!("failed to build guest images");
		return Err(());
	}
	unmount_images_imagefs(cmd, images);
	chmod_readonly_images(cmd, images);
	remove_images_build(cmd, images_build);
	Ok(())
}

fn remove_images(cmd: &mut Cmd, images: &Path) {
	let chmod_result =
		cmd.command_spawn_wait(Command::new("chmod").args(["-R", "u+w"]).arg(images));
	assert!(chmod_result.success());
	unmount_images_imagefs(cmd, images);
	// Recursively delete `$AMBA_DATA_DIR/images/`
	cmd.remove_dir_all(images);
}

fn unmount_images_imagefs(cmd: &mut Cmd, images: &Path) {
	// Unmount `$AMBA_DATA_DIR/images-build/*/imagefs/`
	for entry in cmd.read_dir(images) {
		let entry = entry.unwrap();
		let imagefs = &entry.path().join("imagefs");
		if imagefs.exists() {
			let umount_result = cmd.command_spawn_wait(Command::new("umount").arg(imagefs));
			match umount_result.success() {
				true => tracing::debug!(?imagefs, "unmount successful"),
				false => tracing::debug!(?imagefs, "unmount failed"),
			}
			if imagefs.exists() {
				cmd.remove_dir(imagefs);
			}
		}
	}
}

fn chmod_readonly_images(cmd: &mut Cmd, images: &Path) {
	let chmod_result = cmd.command_spawn_wait(Command::new("chmod").args(["-R", "-w"]).arg(images));
	assert!(chmod_result.success());
}

fn remove_images_build(cmd: &mut Cmd, images_build: &Path) {
	// Recursively chmod+w any nix-built linux kernel packages
	for entry_src in cmd.read_dir(images_build) {
		let entry_src = entry_src.unwrap();
		let tmp_output = entry_src.path().join(".tmp-output");
		if !tmp_output.exists() {
			continue;
		}
		for entry_tmp in cmd.read_dir(tmp_output) {
			let entry_tmp = entry_tmp.unwrap();
			if entry_tmp
				.file_name()
				.to_string_lossy()
				.starts_with("linux-4.9.3-")
			{
				let chmod_result = cmd.command_spawn_wait(
					Command::new("chmod")
						.args(["-R", "+w"])
						.arg(entry_tmp.path()),
				);
				assert!(chmod_result.success());
			}
		}
	}
	// Recursively delete `$AMBA_DATA_DIR/images-build/`
	// The first recursive remove returns "No such file or directory"
	match cmd.try_remove_dir_all(images_build) {
		Ok(()) => {}
		Err(_) => cmd.try_remove_dir_all(images_build).unwrap(),
	}
}
