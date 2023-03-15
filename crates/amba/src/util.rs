use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

pub fn os_str_to_escaped_ascii(os_str: impl AsRef<OsStr>) -> String {
	let raw = os_str.as_ref().as_bytes();
	let escaped = raw.escape_ascii().collect();
	String::from_utf8(escaped).unwrap()
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn escaped_string_is_ascii() {
		let buf: Vec<u8> = (0..=255u8).chain(0..=255u8).collect();
		let os_str = OsStr::from_bytes(&buf);
		let escaped = os_str_to_escaped_ascii(os_str);
		assert!(escaped.is_ascii());
	}
}
