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

#[derive(Debug)]
pub enum Error {
	GimliError(addr2line::gimli::read::Error),
	IoError(std::io::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::GimliError(e) => write!(f, "{e}"),
			Error::IoError(e) => write!(f, "{e}"),
		}
	}
}

impl error::Error for Error {}

impl From<addr2line::gimli::read::Error> for Error {
	fn from(value: addr2line::gimli::read::Error) -> Self {
		Self::GimliError(value)
	}
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

/// For caching source files loaded into memory. So that source code lines can be fetched without
/// rereading the source code files.
pub struct Context {
	files: HashMap<PathBuf, Vec<u8>>, // Cached source files that have already been opened.
	addr2line_context: addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>,
}

impl Context {
	// filepath to the binary.
	pub fn new(filepath: &Path) -> Result<Self, Error> {
		Ok(Self {
			files: HashMap::new(),
			addr2line_context: create_addr2line_context(filepath),
		})
	}

	/// Returns the line of source code corresponding to `addr`, if location information for `addr`
	/// doesn't exist, None is returned. If the source code cannot be found based on the location
	/// information for `addr`, Some(Err()) is returned. Otherwise Some(Ok(String)) is returned.
	pub fn get_line(&mut self, addr: u64) -> Option<Result<String, Error>> {
		match addr2line(&self.addr2line_context, addr) {
			Ok((file_name, line, _)) => {
				let filepath = Path::new(&file_name);
				self.files
					.entry(filepath.to_owned())
					.or_insert_with(|| fs::read(filepath).unwrap())
					.lines()
					.nth(line as usize - 1)
			}
			Err(_) => None,
		}
	}
}

pub fn addr2line(
	context: &addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>,
	addr: u64,
) -> Result<(String, u32, u32), ()> {
	let mut locs = context.find_frames(addr).unwrap();
	tracing::debug!("addr {:#x} belongs to:", addr);
	match locs.next().unwrap() {
		Some(frame) => {
			let location = frame.location.unwrap();
			tracing::debug!(
				"** Function Frame **\nfunction: {:?}\ndw_die_offset: {:?}\nlocation: {}:{}:{}",
				frame.function.unwrap().demangle().unwrap(),
				frame.dw_die_offset.unwrap(),
				location.file.unwrap(),
				location.line.unwrap(),
				location.column.unwrap()
			);
			Ok((
				location.file.unwrap().to_owned(),
				location.line.unwrap(),
				location.column.unwrap(),
			))
		}
		None => Err(()),
	}
}

fn create_addr2line_context<P>(
	filepath: P,
) -> addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>
where
	P: AsRef<Path> + fmt::Debug,
{
	let contents = fs::read(&filepath).expect(&format!("Could not read file {:?}", filepath));
	let parsed = read::File::parse(&*contents).unwrap();
	addr2line::Context::new(&parsed).unwrap()
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
	fn hello() {
		let binary_filepath = Path::new("../../demos/hello");
		let addr = 0x40112F;

		let mut context = Context::new(binary_filepath).unwrap();
		let line = context.get_line(addr).unwrap();

		assert_eq!(
			line.unwrap(),
			read_line("../../demos/hello.c", 4).unwrap()
		)
	}
}
