use std::{
	collections::HashMap,
	fs::{self, File},
	io::{self, BufRead, BufReader, Error, Lines},
	path::{Path, PathBuf},
	rc::Rc,
};

use addr2line::{
	gimli::{EndianReader, RunTimeEndian},
	object::read,
};

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
				match self.files.get(&filepath.to_owned()) {
					Some(file) => file.lines().nth(line as usize - 1),
					None => {
						let contents = fs::read(filepath).unwrap();
						let x = contents.lines().nth(line as usize - 1);
						self.files.insert(filepath.to_owned(), contents);
						x
					}
				}
			}
			Err(_) => None,
		}
	}
}

pub fn main() -> Result<(), ()> {
	let binary_filepath = Path::new("../../demos/hello");
	let addr = 0x40112F;

	let mut context = Context::new(binary_filepath).unwrap();
	let line = context.get_line(addr).unwrap();

	println!("source code corresponding to addr {addr:#X}:");
	println!("{}", line.unwrap());

	Ok(())
}

pub fn addr2line(
	context: &addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>,
	addr: u64,
) -> Result<(String, u32, u32), ()> {
	let mut locs = context.find_frames(addr).unwrap();
	println!("addr {:#x} belongs to:", addr);
	match locs.next().unwrap() {
		Some(frame) => {
			let location = frame.location.unwrap();
			println!(
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

/// Uses 1-indexed line numbers.
fn read_line<P>(filepath: P, line: usize) -> Result<String, Error>
where
	P: AsRef<Path>,
{
	match read_lines(filepath) {
		Ok(mut lines) => lines.nth(line - 1).unwrap(),
		Err(e) => Err(e),
	}
}

fn read_lines<P>(filename: P) -> io::Result<Lines<BufReader<File>>>
where
	P: AsRef<Path>,
{
	let file = File::open(filename)?;
	Ok(BufReader::new(file).lines())
}

fn create_addr2line_context<P>(
	filepath: P,
) -> addr2line::Context<EndianReader<RunTimeEndian, Rc<[u8]>>>
where
	P: AsRef<Path>,
	P: std::fmt::Debug,
{
	let contents = fs::read(&filepath).expect(&format!("Could not read file {:?}", filepath));
	let parsed = read::File::parse(&*contents).unwrap();
	addr2line::Context::new(&parsed).unwrap()
}
