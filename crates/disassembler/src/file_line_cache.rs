use std::{
	fs, io, iter,
	path::{Path, PathBuf},
};

use elsa::FrozenMap;

use crate::Error;

#[derive(Default)]
pub struct FileLineCache {
	/// Maps filepaths to vector of line start indices in the file and the file content itself.
	files: FrozenMap<PathBuf, Box<(Vec<usize>, String)>>,
}

impl FileLineCache {
	/// Gets a reference to the line at `linenumber` in `filepath`. Caches the content of
	/// `filepath` for future calls to `get`. Returns Ok(None) if the file is read but the line at
	/// `linenumber` doesn't exist. Any error is propagated if the file couldn't be read.
	pub fn get<'a>(&'a self, filepath: &Path, linenumber: u32) -> Result<Option<&'a str>, Error> {
		self.populate(filepath)?;
		let (line_start_indices, content) = &self.files[filepath];
		let ret = (|| {
			let this_line = *line_start_indices.get(linenumber as usize - 1)?;
			let next_line = line_start_indices
				.get(linenumber as usize)
				.map_or(content.len(), |&idx| idx);
			let line_content = &content[this_line..next_line];
			// Strip `"\r\n"` or `"\n"` suffixes if present.
			line_content
				.strip_suffix("\r\n")
				.or_else(|| line_content.strip_suffix('\n'))
				.or(Some(line_content))
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
			// The first line starts at byte 0, and all following lines start 1 past a `'\n'` (This
			// holds even for `"\r\n"` line ends!).
			iter::once(0),
			content
				.bytes()
				.enumerate()
				// allow newline characters to be included in this line instead of being part of
				// next line.
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
