use std::{
	fs::{self, ReadDir},
	io, iter,
	path::Path,
	process::{Command, ExitStatus},
	sync::atomic::{AtomicBool, Ordering},
};

pub struct Cmd {
	_no_construct: (),
}
impl Cmd {
	pub fn get() -> Self {
		static ACQUIRED: AtomicBool = AtomicBool::new(false);
		ACQUIRED
			.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
			.expect("Cmd::get() can only be called once");
		Self { _no_construct: () }
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
