use std::{
	collections::HashMap,
	fmt, fs, io,
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
	Io(#[from] io::Error),
	#[error("{0}")]
	Gimli(#[from] addr2line::gimli::Error),
	#[error("{0}")]
	ObjectRead(#[from] read::Error),
	#[error("{0}")]
	Custom(&'static str),
}

#[derive(Default)]
pub struct FileLineCache {
	/// (line_starts, Cached source files that have already been opened).
	files: HashMap<PathBuf, (Vec<usize>, String)>,
}

impl FileLineCache {
	pub fn get<'a>(
		&'a mut self,
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
			Some(&content[this_line..next_line])
		})();

		Ok(ret)
	}

	fn populate(&mut self, filepath: &Path) -> Result<(), io::Error> {
		if self.files.contains_key(filepath) {
			return Ok(());
		}
		let content = fs::read_to_string(filepath)?;
		let mut line_start_indices = Vec::new();
		let mut current_index = 0;
		for line in content.lines() {
			line_start_indices.push(current_index);
			current_index += line.len();
		}
		self.files
			.insert(filepath.to_owned(), (line_start_indices, content));
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
	// filepath to the binary.
	pub fn new(filepath: &Path) -> Result<Self, Error> {
		Ok(Self {
			cache: FileLineCache::default(),
			addr2line_context: Context::create_addr2line_context(filepath)?,
		})
	}

	/// Returns the line of source code corresponding to `addr`, if location information for `addr`
	/// exist, Ok(Some(String)) is returned, if location information doesn't exist in debug
	/// information, Ok(None) is returned, otherwise any errors are propagated as our `Error` enum.
	pub fn get_source_line(&mut self, addr: u64) -> Result<Option<&str>, Error> {
		let line_info = self.addr2loc(addr)?;
		let Some((file_name, line, _)) = line_info else { return Ok(None); };

		let filepath = Path::new(&file_name);
		let res = self.cache.get(filepath, line)?;
		Ok(res)
	}

	pub fn get_source_lines(
		&mut self,
		probe_low: u64,
		probe_high: u64,
	) -> Result<Vec<(u64, u64, addr2line::Location<'_>, String)>, Error> {
		let mut loc_range_iter = self
			.addr2line_context
			.find_location_range(probe_low, probe_high)?
			.peekable();

		if let Some((_, _, loc)) = loc_range_iter.peek() {
			if loc.file.is_none() {
				return Err(Error::Custom(
					"Debug data has no source file reference. Stripped?",
				));
			}
			if loc.line.is_none() {
				return Err(Error::Custom(
					"Debug data has no source code line reference.",
				));
			}
			self.cache
				.get(loc.file.unwrap().as_ref(), loc.line.unwrap())?;
		}

		let mut res = Vec::new();
		for (start_addr, size, loc) in loc_range_iter {
			let item = self
				.cache
				.get(loc.file.unwrap().as_ref(), loc.line.unwrap())?;
			if let Some(line) = item {
				res.push((start_addr, size, loc, line.to_owned()));
			}
		}
		Ok(res)
	}

	/// Get the `Ok(Some((filepath, line, column))` corresponding to an address. If no location
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

	#[test]
	#[ignore]
	fn locs() {
		let binary_filepath = Path::new("../../demos/hello");
		let low = 0x401180;
		let high = 0x401190;
		let mut context = Context::new(binary_filepath).unwrap();
		let res = context.get_source_lines(low, high).unwrap();
		let mut res2 = Vec::new();
		res2 = res
			.iter()
			.map(|(start, size, _, line)| (start, size, line.clone()))
			.collect();
		dbg!(res2);
	}
}
