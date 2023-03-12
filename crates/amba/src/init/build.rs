//! Build guest images locally

use std::{path::Path, process::Command};

use crate::{cmd::Cmd, init::InitStrategy, AMBA_BUILD_GUEST_IMAGES_SCRIPT};

/// Build guest images locally
pub struct InitBuild {
	_no_copy: (),
}

impl InitStrategy for InitBuild {
	fn new() -> Box<Self> {
		Box::new(Self { _no_copy: () })
	}

	/// The version string of the `InitBuild` strategy is the script that builds
	/// the guest images.
	fn version(&self) -> String {
		format!("built using {AMBA_BUILD_GUEST_IMAGES_SCRIPT}\n")
	}

	fn init(self: Box<Self>, cmd: &mut Cmd, data_dir: &Path) -> Result<(), ()> {
		tracing::info!("building guest images");
		let images = &data_dir.join("images");
		let images_build = &data_dir.join("images-build");
		let build_result = cmd.command_spawn_wait(
			Command::new(AMBA_BUILD_GUEST_IMAGES_SCRIPT).args([images_build, images]),
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
}

pub fn unmount_images_imagefs(cmd: &mut Cmd, images: &Path) {
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

pub fn remove_images_build(cmd: &mut Cmd, images_build: &Path) {
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
