use std::{
	fs, io, iter,
	path::{Path, PathBuf},
};

use elsa::FrozenMap;

#[derive(Default)]
pub struct FileLineCache {
	/// Maps filepaths to vector of line start indices in the file and the file content itself.
	files: FrozenMap<PathBuf, Box<Result<(Vec<usize>, String), io::Error>>>,
}

impl FileLineCache {
	/// Gets a reference to the line at `linenumber` in `filepath`. Caches the content of
	/// `filepath` for future calls to `get`. Returns Ok(None) if the file is read but the line at
	/// `linenumber` doesn't exist. Any error is propagated if the file couldn't be read.
	pub fn get<'a>(
		&'a self,
		filepath: &Path,
		linenumber: u32,
	) -> Result<Option<&'a str>, crate::Error> {
		let (line_start_indices, content) = self
			.get_or_populate(filepath)
			.map_err(|()| crate::Error::PopulatingFileLineCache)?;

		let this_line: usize = match line_start_indices.get(linenumber as usize - 1) {
			Some(&this_line) => this_line,
			None => return Ok(None),
		};
		let next_line: usize = match line_start_indices.get(linenumber as usize) {
			Some(&next_line) => next_line,
			None => content.len(),
		};
		let line_content = &content[this_line..next_line];

		// Strip `"\r\n"` or `"\n"` suffixes if present.
		Ok(line_content
			.strip_suffix("\r\n")
			.or_else(|| line_content.strip_suffix('\n'))
			.or(Some(line_content)))
	}

	fn get_or_populate<'a>(&'a self, filepath: &Path) -> Result<&'a (Vec<usize>, String), ()> {
		// `elsa::FrozenMap` does not have a `get_or_insert`/`entry` API.
		match self.files.get(filepath) {
			Some(Ok(value)) => Ok(value),
			Some(Err(_)) => Err(()),
			None => match self.files.insert(
				filepath.to_owned(),
				Box::new(Self::populate(filepath)),
			) {
				Ok(value) => Ok(value),
				Err(err) => {
					tracing::error!(
						?filepath,
						?err,
						"Could not populate FileLineCache"
					);
					Err(())
				}
			},
		}
	}

	/// Caches the content of file at `filepath` if not already cached. `std::io::Error` is
	/// propagated if file couldn't be read.
	fn populate(filepath: &Path) -> Result<(Vec<usize>, String), io::Error> {
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
		Ok((line_start_indices, content))
	}
}
