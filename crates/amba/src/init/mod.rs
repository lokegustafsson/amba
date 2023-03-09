use std::{path::Path, process::Command};

use crate::{cmd::Cmd, InitArgs};

mod build;
mod download;

pub fn init(
	cmd: &mut Cmd,
	data_dir: &Path,
	InitArgs { force, download }: InitArgs,
) -> Result<(), ()> {
	let initializer: Box<dyn InitStrategy> = match download {
		true => download::InitDownload::new(),
		false => build::InitBuild::new(),
	};
	let new_version = initializer.version(cmd);
	let version_file = &data_dir.join("version.txt");
	{
		let old_version = version_file
			.exists()
			.then(|| String::from_utf8(cmd.read(version_file)).unwrap());

		if !force && old_version.as_ref() == Some(&new_version) {
			tracing::info!("guest images already up to date; force rebuild with --force");
			return Ok(());
		}
	}

	{
		version_file.exists().then(|| cmd.remove(version_file));
		let images = &data_dir.join("images");
		let images_build = &data_dir.join("images-build");
		if images.exists() {
			remove_images(cmd, images);
		}
		if images_build.exists() {
			build::remove_images_build(cmd, images_build);
		}
	}

	initializer.init(cmd, data_dir)?;
	assert!(data_dir
		.join("images/ubuntu-22.04-x86_64/image.json")
		.exists());
	assert!(data_dir
		.join("images/ubuntu-22.04-x86_64/image.raw.s2e")
		.exists());
	assert!(data_dir
		.join("images/ubuntu-22.04-x86_64/image.raw.s2e.ready")
		.exists());

	cmd.write(version_file, new_version);
	Ok(())
}
trait InitStrategy {
	fn new() -> Box<Self>
	where
		Self: Sized;
	fn version(&self, cmd: &mut Cmd) -> String;
	fn init(self: Box<Self>, cmd: &mut Cmd, data_dir: &Path) -> Result<(), ()>;
}

fn remove_images(cmd: &mut Cmd, images: &Path) {
	let chmod_result =
		cmd.command_spawn_wait(Command::new("chmod").args(["-R", "u+w"]).arg(images));
	assert!(chmod_result.success());
	build::unmount_images_imagefs(cmd, images);
	// Recursively delete `$AMBA_DATA_DIR/images/`
	cmd.remove_dir_all(images);
}