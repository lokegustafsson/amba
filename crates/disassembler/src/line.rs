use std::{
	fmt, fs, io, iter,
	path::{Path, PathBuf},
	rc::Rc,
};

use addr2line::{
	gimli::{EndianReader, RunTimeEndian},
	object::read,
};
use elsa::FrozenMap;
use thiserror::Error;

type Addr2LineContext = addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>;

#[derive(Error, Debug)]
pub enum Error {
	#[error("{0}")]
	Io(#[from] io::Error),
	#[error("{0}")]
	Gimli(#[from] addr2line::gimli::Error),
	#[error("{0}")]
	ObjectRead(#[from] read::Error),
	#[error("Stripped? Missing debug data: {0}")]
	MissingDebugData(&'static str),
	#[error("Partially stripped? A weird subset of debug data is missing")]
	WeirdDebugData,
}

#[derive(Default)]
pub struct FileLineCache {
	/// Maps filepaths to vector of line start indices in the file and the file content itself.
	files: FrozenMap<PathBuf, Box<(Vec<usize>, String)>>,
}

impl FileLineCache {
	/// Gets a reference to the line at `linenumber` in `filepath`. Caches the content of
	/// `filepath` for future calls to `get`. Returns Ok(None) if the file is read but the line at
	/// `linenumber` doesn't exist. Any `std::io::Error` is propagated if the file couldn't be
	/// read.
	pub fn get<'a>(
		&'a self,
		filepath: &Path,
		linenumber: u32,
	) -> Result<Option<&'a str>, io::Error> {
		self.populate(filepath)?;
		let (line_start_indices, content) = &self.files[filepath];
		let ret = (|| {
			let this_line = *line_start_indices.get(linenumber as usize - 1)?;
			let next_line = line_start_indices
				.get(linenumber as usize)
				.map_or(content.len(), |&idx| idx);
			let ret = &content[this_line..next_line];
			Some(
				if ret.len() >= 2 && &ret[(ret.len() - 2)..] == "\r\n" {
					&ret[..(ret.len() - 2)]
				} else if ret.len() >= 1 && &ret[(ret.len() - 1)..] == "\n" {
					&ret[..(ret.len() - 1)]
				} else {
					ret
				},
			)
		})();

		Ok(ret)
	}

	/// Caches the content of file at `filepath` if not already cached. `std::io::Error` is
	/// propagated if file couldn't be read.
	fn populate(&self, filepath: &Path) -> Result<(), io::Error> {
		if self.files.get(filepath).is_some() {
			return Ok(());
		}
		let content = fs::read_to_string(filepath)?;
		let line_start_indices = Iterator::chain(
			// zero out line number 0, so we can use use 1-indexed linenumbers.
			iter::once(0),
			content
				.as_bytes()
				.iter()
				.copied()
				.enumerate()
				.filter_map(|(i, ch)| (ch == b'\n').then_some(i + 1)),
		)
		.collect();
		self.files.insert(
			filepath.to_owned(),
			Box::new((line_start_indices, content)),
		);
		Ok(())
	}
}

/// For caching source files loaded into memory. So that source code lines can be fetched without
/// rereading the source code files.
pub struct Context {
	cache: FileLineCache, // Cached source files that have already been opened.
	addr2line_context: Addr2LineContext,
}

impl Context {
	/// `filepath` is the path to the binary.
	pub fn new(filepath: &Path) -> Result<Self, Error> {
		Ok(Self {
			cache: FileLineCache::default(),
			addr2line_context: Context::create_addr2line_context(filepath)?,
		})
	}

	/// Returns the line of source code corresponding to `addr`, if location information for `addr`
	/// exist, Ok(Some(&str)) is returned, if location information doesn't exist in debug
	/// information, Ok(None) is returned, otherwise any errors are propagated as our `Error` enum.
	pub fn get_source_line(&self, addr: u64) -> Result<Option<&str>, Error> {
		let line_info = self.addr2loc(addr)?;
		let Some((file_name, line, _)) = line_info else { return Ok(None); };

		let filepath = Path::new(&file_name);
		let res = self.cache.get(filepath, line)?;
		Ok(res)
	}

	/// Returns source code line information for a virtual adress range in binary if the sources
	/// exist. an item in the resulting `Vec` is (start_virt_addr, size_in_bytes,
	/// addr2line::Location, &line_of_source_code).
	pub fn get_source_lines(
		&self,
		probe_low: u64,
		probe_high: u64,
	) -> Result<Vec<(u64, u64, addr2line::Location<'_>, &str)>, Error> {
		let mut loc_range_iter = self
			.addr2line_context
			.find_location_range(probe_low, probe_high)?
			.peekable();

		if let Some((_, _, loc)) = loc_range_iter.peek() {
			match (loc.file, loc.line) {
				(None, _) => {
					return Err(Error::MissingDebugData("source file reference"));
				}
				(_, None) => {
					return Err(Error::MissingDebugData("source line reference"));
				}
				(Some(file), Some(line)) => {
					self.cache.get(file.as_ref(), line)?;
				}
			}
		}

		let mut res = Vec::new();
		for (start_addr, size, loc) in loc_range_iter {
			let item = self.cache.get(
				loc.file.ok_or(Error::WeirdDebugData)?.as_ref(),
				loc.line.ok_or(Error::WeirdDebugData)?,
			)?;
			if let Some(line) = item {
				res.push((start_addr, size, loc, line));
			}
		}
		Ok(res)
	}

	/// Get the `Ok(Some((filepath, line, column)))` corresponding to an address. If no location
	/// information for the `addr` is found, `Ok(None)` is returned. Other errors from addr2line is
	/// otherwise propagated and wrapped in our `Error`.
	pub fn addr2loc(&self, probe: u64) -> Result<Option<(String, u32, u32)>, Error> {
		let mut locs = self.addr2line_context.find_frames(probe)?;
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
	// NOTE: Ignored because uses relative paths and the compiled binary can have different
	// addresses. So it will not always pass...
	fn hello() {
		let binary_filepath = Path::new("../../demos/hello");
		let addr = 0x401134;

		let context = Context::new(binary_filepath).unwrap();
		let line = context.get_source_line(addr).unwrap().unwrap().to_owned();

		assert_eq!(line, read_line("../../demos/hello.c", 4).unwrap());
	}

	#[test]
	#[ignore]
	fn locs() {
		let binary_filepath = Path::new("../../demos/hello");
		let low = 0x401126;
		let high = 0x40113F;
		let context = Context::new(binary_filepath).unwrap();
		let res: Vec<_> = context
			.get_source_lines(low, high)
			.unwrap()
			.into_iter()
			.map(|(start, size, _, line)| (start, size, line))
			.collect();
		dbg!(res);
	}
}
