use std::{
	fs::{self, ReadDir},
	io, iter,
	marker::PhantomData,
	path::Path,
	process::{Command, ExitStatus},
};

pub struct Cmd {
	_marker: PhantomData<()>,
}
impl Cmd {
	pub fn new() -> Self {
		Self {
			_marker: PhantomData,
		}
	}

	pub fn command_spawn_wait(&mut self, command: &mut Command) -> ExitStatus {
		tracing::trace!(
		  cwd = ?command.get_current_dir(),
		  env = ?command.get_envs().collect::<Vec<_>>(),
		  args = ?iter::once(command.get_program()).chain(command.get_args()).collect::<Vec<_>>()
		);
		command.spawn().unwrap().wait().unwrap()
	}

	pub fn read_dir(&mut self, dir: impl AsRef<Path>) -> ReadDir {
		let dir = dir.as_ref();
		tracing::trace!(?dir, "read_dir");
		fs::read_dir(dir).unwrap()
	}

	pub fn remove_dir(&mut self, dir: impl AsRef<Path>) {
		let dir = dir.as_ref();
		tracing::trace!(?dir, "remove_dir");
		fs::remove_dir(dir).unwrap()
	}

	pub fn remove_dir_all(&mut self, dir: impl AsRef<Path>) {
		self.try_remove_dir_all(dir).unwrap()
	}

	pub fn try_remove_dir_all(&mut self, dir: impl AsRef<Path>) -> io::Result<()> {
		let dir = dir.as_ref();
		tracing::trace!(?dir, "remove_dir_all");
		fs::remove_dir_all(dir)
	}

	pub fn create_dir_all(&mut self, dir: impl AsRef<Path>) {
		let dir = dir.as_ref();
		tracing::trace!(?dir, "create_dir_all");
		fs::create_dir_all(dir).unwrap()
	}

	pub fn write(&mut self, file: impl AsRef<Path>, content: impl AsRef<[u8]>) {
		let file = file.as_ref();
		tracing::trace!(?file, "write_file");
		fs::write(file, content).unwrap()
	}
}
