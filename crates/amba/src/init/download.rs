use std::{
	io::{self, Read},
	path::Path,
};

use url::Url;

use crate::cmd::Cmd;

pub fn force_init_download(cmd: &mut Cmd, data_dir: &Path) -> Result<(), ()> {
	// From the URL <https://drive.google.com/file/d/102EgrujJE5Pzlg98qe3twLNIeMz5MkJQ/view>
	let fileid = "102EgrujJE5Pzlg98qe3twLNIeMz5MkJQ";
	let agent = ureq::Agent::new();
	let confirm_uuid = {
		let confirm_page_html = cmd
			.http_get(
				&agent,
				Url::parse(&format!(
					"https://drive.google.com/uc?export=download&id={fileid}"
				))
				.unwrap(),
			)
			.into_string()
			.unwrap();
		const REGEX: &str = concat!(
			"confirm=t&amp;uuid=(",
			"[0-9a-f]{8}",
			"-",
			"[0-9a-f]{4}",
			"-",
			"[0-9a-f]{4}",
			"-",
			"[0-9a-f]{4}",
			"-",
			"[0-9a-f]{12}",
			")"
		);
		regex::Regex::new(REGEX)
			.unwrap()
			.captures(&confirm_page_html)
			.unwrap_or_else(|| {
				panic!("found no confirmation uuid in response body:\n{confirm_page_html}\n")
			})
			.get(1)
			.unwrap()
			.as_str()
			.to_owned()
	};
	{
		let resp = cmd.http_get(
			&agent,
			Url::parse(&format!(
					"https://drive.google.com/uc?id={fileid}&export=download&confirm=t&uuid={confirm_uuid}"
				))
			.unwrap(),
		);
		let content_length = resp
			.header("Content-Length")
			.unwrap()
			.parse::<u64>()
			.unwrap();
		let download_read = resp.into_reader();
		let with_progress_read = ProgressReader::new(download_read, content_length);
		let xz_read = xz2::read::XzDecoder::new(with_progress_read);
		let mut tar_read = tar::Archive::new(xz_read);
		tar_read.unpack(data_dir.join("images")).unwrap();
	}
	Ok(())
}

struct ProgressReader<R> {
	inner: R,
	current: u64,
	latest_log: u64,
	total: u64,
}
impl<R: Read> ProgressReader<R> {
	fn new(inner: R, total: u64) -> Self {
		Self {
			inner,
			current: 0,
			latest_log: 0,
			total,
		}
	}
}
impl<R: Read> Read for ProgressReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let ret = self.inner.read(buf);
		if let Ok(bytes) = ret {
			const THRESHOLD: u64 = 64 * 1024 * 1024;
			self.current += bytes as u64;
			if self.current - self.latest_log > THRESHOLD {
				self.latest_log = self.current;
				let progress_mibibytes = self.current / (1024 * 1024);
				let total_mibibytes = self.total / (1024 * 1024);
				tracing::trace!(progress_mibibytes, total_mibibytes);
			}
		}
		ret
	}
}
