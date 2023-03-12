//! Download guest images from Google Drive

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

/// From the URL
/// <https://drive.google.com/file/d/102EgrujJE5Pzlg98qe3twLNIeMz5MkJQ/view>
const GOOGLE_DRIVE_FILE_ID: &str = "102EgrujJE5Pzlg98qe3twLNIeMz5MkJQ";

/// Download guest images from Google Drive
pub struct InitDownload {
	_no_copy: (),
}

impl InitStrategy for InitDownload {
	fn new() -> Box<Self> {
		Box::new(Self { _no_copy: () })
	}

	/// The version string of the `InitDownload` strategy is the Google Drive url of
	/// the tarball.
	fn version(&self) -> String {
		format!(
			"downloaded from https://drive.google.com/file/d/{}/view\n",
			GOOGLE_DRIVE_FILE_ID
		)
	}

	/// Download and extract the guest images using the undocumented token-less
	/// Google Drive API (the API used by not-logged-in humans for files >100MB in
	/// the web browser)
	fn init(self: Box<Self>, cmd: &mut Cmd, data_dir: &Path) -> Result<(), ()> {
		tracing::info!("downloading guest images");
		// Set up a HTTP client with persistent cookies.
		let jar = Arc::new(Jar::default());
		let client = ClientBuilder::new()
			.redirect(redirect::Policy::none())
			.cookie_provider(Arc::clone(&jar))
			.build()
			.unwrap();
		// Acquire a cookie token, possibly helping the download later. (not sure)
		{
			let drive_url = Url::parse("https://drive.google.com").unwrap();
			assert!(jar.cookies(&drive_url).is_none());
			let resp = cmd.http(
				&client,
				Method::GET,
				Url::parse(&format!(
					"https://drive.google.com/file/d/{}/view",
					GOOGLE_DRIVE_FILE_ID
				))
				.unwrap(),
				&[],
			);
			assert!(resp.status().is_success());
			assert!(jar.cookies(&drive_url).is_some());
		}
		// Acquire the uuid confirming that we still want to download the file after seeing:
		// > ubuntu-22.04-x86_64.tar.xz (3.1G) is too large for Google to scan for viruses.
		// > Would you still like to download this file?
		let confirm_uuid = {
			let confirm_page_html = cmd
				.http(
					&client,
					Method::GET,
					Url::parse(&format!(
						"https://drive.google.com/uc?export=download&id={}",
						GOOGLE_DRIVE_FILE_ID
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
		// Download the file in a streaming manner, using readers
		// 1. [`ResumingReader`] (download, supporting resume after timeouts)
		// 2. [`ProgressReader`] (log progress periodically)
		// 3. [`xz2::read::XzDecoder`] (uncompress `.xz`)
		// 4. [`tar::Archive::new`] (unpack `.tar`)
		{
			let download_read = ResumingReader::new(move |offset| {
				cmd.http(
					&client,
					Method::POST,
					Url::parse(&format!(
						"https://drive.google.com/uc?id={}&export=download&confirm=t&uuid={}",
						GOOGLE_DRIVE_FILE_ID, confirm_uuid
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

/// A reader wrapper around [`reqwest::Response`] that restarts the request with
/// the `Range: bytes=<start>-` HTTP header on [`reqwest::Error::is_timeout`].
struct ResumingReader<F: FnMut(u64) -> Response> {
	start_download: F,
	inner: Response,
	current: u64,
	content_length: u64,
}

impl<F: FnMut(u64) -> Response> ResumingReader<F> {
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

impl<F: FnMut(u64) -> Response> Read for ResumingReader<F> {
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
		let rerr: &reqwest::Error = match err.get_ref().and_then(|e| e.downcast_ref()) {
			Some(rerr) => rerr,
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

/// A generic reader wrapper that tracks the current read length and logs every
/// 64MB of progress.
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
