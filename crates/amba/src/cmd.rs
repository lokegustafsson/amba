use std::{
	collections::{hash_map, HashMap},
	fs::{self, ReadDir},
	io, iter, mem,
	path::Path,
	process::{self, Child, Command, ExitStatus, Output, Stdio},
	sync::{
		atomic::{AtomicBool, Ordering},
		Mutex, TryLockError,
	},
	thread,
	time::Duration,
};

static CHILDREN: Mutex<Option<HashMap<u32, Child>>> = Mutex::new(None);

pub struct Cmd {
	_no_construct: (),
}
impl Cmd {
	pub fn get() -> Self {
		static ACQUIRED: AtomicBool = AtomicBool::new(false);
		ACQUIRED
			.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
			.expect("Cmd::get() can only be called once");
		ctrlc::set_handler(ctrlc_handler).unwrap();
		Self { _no_construct: () }
	}

	pub fn command_spawn_wait(&mut self, command: &mut Command) -> ExitStatus {
		tracing::trace!(
			cwd = ?command.get_current_dir(),
			env = ?command.get_envs().collect::<Vec<_>>(),
			args = ?iter::once(command.get_program()).chain(command.get_args()).collect::<Vec<_>>()
		);
		safe_wait(command.spawn().unwrap()).wait().unwrap()
	}

	pub fn command_capture_stdout(&mut self, command: &mut Command) -> Result<Vec<u8>, Vec<u8>> {
		tracing::trace!(
			cwd = ?command.get_current_dir(),
			env = ?command.get_envs().collect::<Vec<_>>(),
			args = ?iter::once(command.get_program()).chain(command.get_args()).collect::<Vec<_>>(),
			"capturing stdout"
		);
		let Output {
			status,
			stdout,
			stderr,
		} = safe_wait(
			command
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.stderr(Stdio::inherit())
				.spawn()
				.unwrap(),
		)
		.wait_with_output()
		.unwrap();
		assert!(
			stderr.is_empty(),
			"stderr: '{}'",
			String::from_utf8_lossy(&stderr)
		);
		match status.success() {
			true => Ok(stdout),
			false => Err(stdout),
		}
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

	pub fn read(&mut self, file: impl AsRef<Path>) -> Vec<u8> {
		let file = file.as_ref();
		tracing::trace!(?file, "read_file");
		fs::read(file).unwrap()
	}

	pub fn copy(&mut self, file: impl AsRef<Path>, target: impl AsRef<Path>) {
		let file = file.as_ref();
		let target = target.as_ref();
		tracing::trace!(?file, ?target, "copy_file");
		fs::write(target, fs::read(file).unwrap()).unwrap()
	}
}

fn safe_wait(child: Child) -> Child {
	let id = child.id();
	{
		let mut slot = CHILDREN.try_lock().unwrap();
		if slot.is_none() {
			*slot = Some(HashMap::default());
		}
		slot.as_mut().unwrap().insert(id, child);
	}
	loop {
		{
			let mut guard = CHILDREN.try_lock().unwrap();
			let mut entry = match guard.as_mut().unwrap().entry(id) {
				hash_map::Entry::Occupied(occupied) => occupied,
				hash_map::Entry::Vacant(_) => unreachable!(),
			};
			match entry.get_mut().try_wait().unwrap() {
				Some(_) => return entry.remove(),
				None => {}
			}
		}
		thread::sleep(Duration::from_millis(50));
	}
}

pub fn ctrlc_handler() {
	println!();
	let mut guard = match CHILDREN.try_lock() {
		Ok(guard) => guard,
		Err(TryLockError::WouldBlock) => {
			tracing::error!("child process map is locked; we cannot kill our children and must leak them instead");
			process::exit(2);
		}
		Err(TryLockError::Poisoned(poison)) => {
			panic!("unexpected poison: {poison:#?}");
		}
	};
	if let Some(map) = mem::take(&mut *guard) {
		for (pid, mut child) in map {
			match child.kill() {
				Ok(()) => tracing::info!(pid, "handling SIGINT; killing child process"),
				Err(err) if err.kind() == io::ErrorKind::InvalidInput => {
					tracing::info!(
						pid,
						"handling SIGINT; child process already exited"
					)
				}
				Err(err) => panic!("unexpected io error: {:#?}", err),
			}
		}
	}
	Box::leak(Box::new(guard));
	process::exit(1);
}
