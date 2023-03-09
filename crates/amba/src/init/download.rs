use std::{
	io::{self, Read},
	path::Path,
	sync::Arc,
};

use reqwest::{
	blocking::{ClientBuilder, Response},
	cookie::{CookieStore, Jar},
	header::{self, HeaderValue},
	redirect, Method,
};
use url::Url;

use crate::{cmd::Cmd, init::InitStrategy};

pub struct InitDownload {
	file_id: &'static str,
}
impl InitStrategy for InitDownload {
	fn new() -> Box<Self> {
		Box::new(Self {
			// From the URL <https://drive.google.com/file/d/102EgrujJE5Pzlg98qe3twLNIeMz5MkJQ/view>
			file_id: "102EgrujJE5Pzlg98qe3twLNIeMz5MkJQ",
		})
	}

	fn version(&self, _: &mut Cmd) -> String {
		format!("{}\n", self.file_id)
	}

	fn init(self: Box<Self>, cmd: &mut Cmd, data_dir: &Path) -> Result<(), ()> {
		tracing::info!("downloading guest images");
		let jar = Arc::new(Jar::default());
		let client = ClientBuilder::new()
			.redirect(redirect::Policy::none())
			.cookie_provider(Arc::clone(&jar))
			.build()
			.unwrap();
		{
			// This request sets an authenication token that is required to not make the
			// download time out later on
			let drive_url = Url::parse("https://drive.google.com").unwrap();
			assert!(jar.cookies(&drive_url).is_none());
			let resp = cmd.http(
				&client,
				Method::GET,
				Url::parse(&format!(
					"https://drive.google.com/file/d/{}/view",
					self.file_id
				))
				.unwrap(),
				&[],
			);
			assert!(resp.status().is_success());
			assert!(jar.cookies(&drive_url).is_some());
		}
		let confirm_uuid = {
			let confirm_page_html = cmd
				.http(
					&client,
					Method::GET,
					Url::parse(&format!(
						"https://drive.google.com/uc?export=download&id={}",
						self.file_id
					))
					.unwrap(),
					&[],
				)
				.text()
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
			let download_read = RestartingReader::new(move |offset| {
				cmd.http(
					&client,
					Method::POST,
					Url::parse(&format!(
						"https://drive.google.com/uc?id={}&export=download&confirm=t&uuid={}",
						self.file_id, confirm_uuid
					))
					.unwrap(),
					&[(
						header::RANGE,
						HeaderValue::try_from(format!("bytes={offset}-")).unwrap(),
					)],
				)
			});
			let content_length = download_read.content_length;
			let with_progress_read = ProgressReader::new(download_read, content_length);
			let xz_read = xz2::read::XzDecoder::new(with_progress_read);
			let mut tar_read = tar::Archive::new(xz_read);
			tar_read.unpack(data_dir.join("images")).unwrap();
		}
		Ok(())
	}
}

struct RestartingReader<F: FnMut(u64) -> Response> {
	start_download: F,
	inner: Response,
	current: u64,
	content_length: u64,
}
impl<F: FnMut(u64) -> Response> RestartingReader<F> {
	fn new(mut start_download: F) -> Self {
		let inner = start_download(0);
		let content_length = inner
			.headers()
			.get("Content-Length")
			.unwrap()
			.to_str()
			.unwrap()
			.parse::<u64>()
			.unwrap();
		Self {
			start_download,
			inner,
			current: 0,
			content_length,
		}
	}
}
impl<F: FnMut(u64) -> Response> Read for RestartingReader<F> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let err: io::Error = match self.inner.read(buf) {
			Ok(len) => {
				self.current += len as u64;
				return Ok(len);
			}
			Err(err) => err,
		};
		if err.kind() != io::ErrorKind::Other {
			return Err(err);
		}
		let rerr: &reqwest::Error = match err.get_ref() {
			Some(rerr) => match rerr.downcast_ref() {
				Some(rerr) => rerr,
				None => return Err(err),
			},
			None => return Err(err),
		};
		assert!(rerr.is_timeout());
		tracing::debug!(bytes = self.current, "restarting download at");
		self.inner = (self.start_download)(self.current);
		Err(io::Error::new(
			io::ErrorKind::Interrupted,
			"had to restart download due to timeout",
		))
	}
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
				let progress_mb = self.current / (1024 * 1024);
				let total_mb = self.total / (1024 * 1024);
				tracing::trace!(progress_mb, total_mb);
			}
		}
		ret
	}
}
