use std::{
	collections::HashMap,
	fmt, fs,
	io::BufRead,
	path::{Path, PathBuf},
	rc::Rc,
};

use addr2line::{
	gimli::{EndianReader, RunTimeEndian},
	object::read,
};
use thiserror::Error;

type Addr2LineContext = addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>;

#[derive(Error, Debug)]
pub enum Error {
	#[error("{0}")]
	Io(#[from] std::io::Error),
	#[error("{0}")]
	Gimli(#[from] addr2line::gimli::Error),
	#[error("{0}")]
	ObjectRead(#[from] read::Error),
}

/// For caching source files loaded into memory. So that source code lines can be fetched without
/// rereading the source code files.
pub struct Context {
	files: HashMap<PathBuf, Vec<u8>>, // Cached source files that have already been opened.
	addr2line_context: Addr2LineContext,
}

impl Context {
	// filepath to the binary.
	pub fn new(filepath: &Path) -> Result<Self, Error> {
		Ok(Self {
			files: HashMap::new(),
			addr2line_context: Context::create_addr2line_context(filepath)?,
		})
	}

	/// Returns the line of source code corresponding to `addr`, if location information for `addr`
	/// exist, Ok(Some(String)) is returned, if location information doesn't exist in debug
	/// information, Ok(None) is returned, otherwise any errors are propagated as our `Error` enum.
	pub fn get_source_line(&mut self, addr: u64) -> Result<Option<String>, Error> {
		let line_info = self.addr2loc(addr)?;
		let Some((file_name, line, _)) = line_info else { return Ok(None); };

		let filepath = Path::new(&file_name);
		let file = if let Some(file) = self.files.get(filepath) {
			file
		} else {
			let path = filepath.to_path_buf();
			let data = fs::read(filepath)?;

			self.files.entry(path).or_insert(data)
		};
		let res = file.lines().nth(line as usize - 1).transpose()?;

		Ok(res)
	}

	/// Get the `Ok(Some((filepath, line, column))` corresponding to an address. If no location
	/// information for the `addr` is found, `Ok(None)` is returned. Other errors from addr2line is
	/// otherwise propagated and wrapped in our `Error`.
	pub fn addr2loc(&self, addr: u64) -> Result<Option<(String, u32, u32)>, Error> {
		let mut locs = self.addr2line_context.find_frames(addr)?;
		let frame = locs.next()?.and_then(|frame| {
			let location = frame.location?;
			Some((
				location.file?.to_owned(),
				location.line?,
				location.column?,
			))
		});
		Ok(frame)
	}

	fn create_addr2line_context<P>(filepath: P) -> Result<Addr2LineContext, Error>
	where
		P: AsRef<Path> + fmt::Debug,
	{
		let contents = fs::read(&filepath)?;
		let parsed = read::File::parse(&*contents)?;
		Ok(addr2line::Context::new(&parsed)?)
	}
}

#[cfg(test)]
mod test {
	use std::{
		fs::File,
		io::{self, BufRead, BufReader, Error, Lines},
		path::Path,
	};

	use crate::*;

	/// Uses 1-indexed line numbers.
	fn read_line<P>(filepath: P, line: usize) -> Result<String, Error>
	where
		P: AsRef<Path>,
	{
		read_lines(filepath).map(|mut ls| ls.nth(line - 1).unwrap())?
	}

	fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>>
	where
		P: AsRef<Path>,
	{
		let file = File::open(filename)?;
		Ok(BufReader::new(file).lines())
	}

	#[test]
	#[ignore]
	// NOTE: Ignored because uses realtive paths and the compiled binary can have different
	// addresses. So it will not always pass...
	fn hello() {
		let binary_filepath = Path::new("../../demos/hello");
		let addr = 0x401180;

		let mut context = Context::new(binary_filepath).unwrap();
		let line = context.get_source_line(addr).unwrap();

		assert_eq!(
			line.unwrap(),
			read_line("../../demos/hello.c", 4).unwrap()
		);
	}
}
