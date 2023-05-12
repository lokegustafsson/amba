use std::{
	fs, io,
	path::{Path, PathBuf},
	rc::Rc,
};

use addr2line::{
	gimli::{EndianReader, RunTimeEndian},
	object::read::{Error as ObjectReadError, File as ObjectFile},
};
use capstone::prelude::*;
use thiserror::Error;

use crate::file_line_cache::FileLineCache;

mod file_line_cache;

#[derive(Error, Debug)]
pub enum Error {
	#[error("{0}")]
	Io(#[from] io::Error),
	#[error("{0}")]
	Gimli(#[from] addr2line::gimli::Error),
	#[error("{0}")]
	ObjectRead(#[from] ObjectReadError),
	#[error("Stripped? Missing debug data: {0}")]
	MissingDebugData(&'static str),
	#[error("Partially stripped? A weird subset of debug data is missing")]
	WeirdDebugData,
	#[error("Could not populate FileLineCache")]
	PopulatingFileLineCache,
}

pub struct DisasmContext {
	recipe_dir: PathBuf,
	file_line_cache: FileLineCache,
	addr2line_context: addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>,
	capstone: Capstone,
}

impl DisasmContext {
	/// `filepath` is the path to the binary.
	pub fn new(filepath: &Path, recipe_dir: &Path) -> Result<Self, Error> {
		Ok(Self {
			recipe_dir: recipe_dir.to_owned(),
			file_line_cache: FileLineCache::default(),
			addr2line_context: {
				let contents = fs::read(&filepath)?;
				let parsed = ObjectFile::parse(&*contents)?;
				addr2line::Context::new(&parsed)?
			},
			capstone: Capstone::new()
				.x86()
				.mode(arch::x86::ArchMode::Mode64)
				.syntax(arch::x86::ArchSyntax::Intel)
				.detail(true)
				.build()
				.expect("Failed to create Capstone object"),
		})
	}

	pub fn x64_to_assembly(&self, x64_code: &[u8], start_addr: u64) -> Vec<(usize, String)> {
		let insns = self
			.capstone
			.disasm_all(x64_code, start_addr)
			.expect("Failed to disassemble");
		insns
			.iter()
			.map(|ins| (ins.len(), ins.to_string()))
			.collect()
	}

	pub fn get_function_name(&self, addr: u64) -> Result<String, Error> {
		let mut frames = self.addr2line_context.find_frames(addr)?;
		let mut ret = String::new();
		while let Some(frame) = frames.next()? {
			if !ret.is_empty() {
				ret.push_str(" in ");
			}
			match frame.function {
				Some(name) => {
					let innermost = ret.is_empty();
					ret.push_str(&*name.demangle()?);
					if innermost {
						ret.push_str("()")
					}
				}
				None => ret.push_str("?"),
			}
		}
		if ret.is_empty() {
			ret.push_str("<unknown>");
		}
		Ok(ret)
	}

	/// Returns the line of source code corresponding to `addr`, if location information for `addr`
	/// exist, Ok(Some(&str)) is returned, if location information doesn't exist in debug
	/// information, Ok(None) is returned, otherwise any errors are propagated as our `Error` enum.
	pub fn get_source_line(&self, addr: u64) -> Result<Option<&str>, Error> {
		let line_info = self.addr2loc(addr)?;
		let Some((file_name, line, _)) = line_info else { return Ok(None); };

		let filepath = Path::new(&file_name);
		let alternative_filepath = self.recipe_dir.join(filepath.file_name().unwrap());

		self.file_line_cache
			.get(filepath, line)
			.or_else(|_err| self.file_line_cache.get(&alternative_filepath, line))
	}

	/// Returns source code line information for a virtual adress range in binary if the sources
	/// exist. an item in the resulting `Vec` is `(start_virt_addr, size_in_bytes,
	/// addr2line::Location, &line_of_source_code)`.
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
					return Err(Error::MissingDebugData("Source file reference"));
				}
				(_, None) => {
					return Err(Error::MissingDebugData("Source line reference"));
				}
				(Some(file), Some(line)) => {
					self.file_line_cache.get(file.as_ref(), line)?;
				}
			}
		}

		let mut res = Vec::with_capacity(loc_range_iter.size_hint().0);
		for (start_addr, size, loc) in loc_range_iter {
			let item = self.file_line_cache.get(
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
	fn addr2loc(&self, probe: u64) -> Result<Option<(String, u32, u32)>, Error> {
		let mut locs = self.addr2line_context.find_frames(probe)?;
		let frame = locs.next()?.and_then(|frame| {
			let location = frame.location?;
			Some((
				location.file.unwrap().to_owned(),
				location.line?,
				location.column?,
			))
		});
		Ok(frame)
	}
}

#[cfg(test)]
mod test {
	use std::{
		env, fs,
		fs::File,
		io::{self, BufRead, BufReader, Error, Lines},
		path::{Path, PathBuf},
		process::Command,
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

	fn create_hello_prog() -> (PathBuf, PathBuf) {
		const HELLO_PROG: &str = r#"
		#include <stdio.h>

		int main() {
			puts("Hello world");
		}
		"#;

		let out_dir = env::temp_dir();
		let dest_src_path = Path::new(&out_dir).join("hello.c");
		let dest_bin_path = Path::new(&out_dir).join("hello");
		fs::write(&dest_src_path, HELLO_PROG).unwrap();
		Command::new("gcc")
			.args(&[&dest_src_path.to_string_lossy(), "-gdwarf", "-O0", "-o"])
			.arg(&dest_bin_path)
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status()
			.unwrap();
		(dest_src_path, dest_bin_path)
	}

	#[test]
	fn get_one_source_line() {
		let (source_filepath, binary_filepath) = create_hello_prog();
		const ADDR: u64 = 0x401134;

		let context = DisasmContext::new(
			&binary_filepath,
			source_filepath.parent().unwrap(),
		)
		.unwrap();
		let line = context.get_source_line(ADDR).unwrap().unwrap().to_owned();

		assert_eq!(line, read_line(source_filepath, 5).unwrap());
	}

	#[test]
	fn get_many_source_lines() {
		let (source_filepath, binary_filepath) = create_hello_prog();
		let low = 0x401126;
		let high = 0x40113F;
		let context = DisasmContext::new(
			&binary_filepath,
			source_filepath.parent().unwrap(),
		)
		.unwrap();
		let res: Vec<_> = context
			.get_source_lines(low, high)
			.unwrap()
			.into_iter()
			.map(|(start, size, _, line)| (start, size, line))
			.collect();

		let line_4 = &read_line(&source_filepath, 4).unwrap();
		let line_5 = &read_line(&source_filepath, 5).unwrap();
		let line_6 = &read_line(&source_filepath, 6).unwrap();
		let expected: Vec<(u64, u64, &str)> = vec![
			(0x401126, 4, line_4),
			(0x40112A, 20, line_5),
			(0x40113E, 2, line_6),
		];
		assert_eq!(res, expected);
	}
}
