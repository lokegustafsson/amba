use std::{
	collections::HashMap,
	error, fmt, fs,
	io::BufRead,
	path::{Path, PathBuf},
	rc::Rc,
};

use addr2line::{
	gimli::{EndianReader, RunTimeEndian},
	object::read,
};

type Addr2LineContext = addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Error {
	IoError(std::io::Error),
	GimliError(addr2line::gimli::Error),
	ObjectReadError(read::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::IoError(e) => write!(f, "{e}"),
			Error::GimliError(e) => write!(f, "{e}"),
			Error::ObjectReadError(e) => write!(f, "{e}"),
		}
	}
}

impl error::Error for Error {}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<addr2line::gimli::Error> for Error {
	fn from(value: addr2line::gimli::Error) -> Self {
		Self::GimliError(value)
	}
}

impl From<read::Error> for Error {
	fn from(value: read::Error) -> Self {
		Self::ObjectReadError(value)
	}
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
		let res = match self.addr2loc(addr)? {
			Some((file_name, line, _)) => {
				let filepath = Path::new(&file_name);
				let file = if let Some(file) = self.files.get(filepath) {
					file
				} else {
					self.files
						.insert(filepath.to_path_buf(), fs::read(filepath)?);
					self.files.get(filepath).unwrap()
				};
				file.lines().nth(line as usize - 1).transpose()?
			}
			None => None,
		};
		Ok(res)
	}

	/// Get the `Ok(Some((filepath, line, column))` corresponding to an address. If no location
	/// information for the `addr` is found, `Ok(None)` is returned. Other errors from addr2line is
	/// otherwise propagated and wrapped in our `Error`.
	pub fn addr2loc(&self, addr: u64) -> Result<Option<(String, u32, u32)>, Error> {
		let mut locs = self.addr2line_context.find_frames(addr)?;
		tracing::debug!("addr {:#x} belongs to:", addr);

		let frame = locs.next()?.and_then(|frame| {
			let location = frame.location?;
			tracing::debug!(
				"** Function Frame **\nfunction: {:?}\ndw_die_offset: {:?}\nlocation: {}:{}:{}",
				frame.function?.demangle().unwrap(),
				frame.dw_die_offset?,
				location.file?,
				location.line?,
				location.column?,
			);
			Some((
				location.file?.to_owned(),
				location.line?,
				location.column?,
			))
		});

		Ok(frame)
	}

	fn create_addr2line_context<P>(
		filepath: P,
	) -> Result<Addr2LineContext, Error>
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
		)
	}
}
