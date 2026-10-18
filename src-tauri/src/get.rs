mod anime;
mod browse;
mod cache;
mod next;
mod schedule;
mod search;
mod types;
mod video;
pub use anime::ani;
pub use browse::browse;
pub use schedule::schedule;
pub use search::search;
pub use types::*;
pub use video::video;

use anyhow::{Context, Error, Result, anyhow};
use std::{io::{ErrorKind, Read}, sync::LazyLock, time::{Duration, Instant}};
use ureq::{Body, BodyReader, http::Response};

const NEXT_BASE_URL: &str = "https://lie.ygopro3.cn";
pub(super) static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
	// Request uncompressed HTML/JSON, while retaining decoding support for
	// servers that still return a Content-Encoding header.
	ureq::Agent::config_builder().accept_encoding("identity").build().into()
});

async fn get(url: &str) -> Result<String, Error> {
	let cache_key: String = format!("GET:{}", url);
	if let Some(content) = cache::get(cache_key.clone()) {
		return Ok(content);
	}
	let content: String = fetch_text(url).await?;
	cache::set(cache_key, content.clone());
	Ok(content)
}

async fn fetch_text(url: &str) -> Result<String> {
	let started: Instant = Instant::now();
	let budget: Duration = Duration::from_secs(30);
	let mut attempt: u32 = 0;
	loop {
		attempt += 1;
		let remaining: Duration = budget.saturating_sub(started.elapsed());
		if remaining.is_zero() {
			return Err(anyhow!("获取番剧页面超时（总时限 30 秒）"));
		}
		let result: Result<String> = AGENT.get(url)
			.config().timeout_global(Some(remaining)).build()
			.call().context("请求番剧页面失败")
			.and_then(|response: Response<Body>| read_response(response).context("读取番剧页面响应失败"));
		match result {
			Ok(content) => return Ok(content),
			Err(error) => {
				let delay: Duration = Duration::from_millis(300 * u64::from(attempt));
				if attempt >= 3 || !retryable_read(&error) || started.elapsed().saturating_add(delay) >= budget {
					return Err(error).with_context(|| format!("获取番剧页面失败（已尝试 {attempt} 次）"));
				}
				tokio::time::sleep(delay).await;
			}
		}
	}
}

fn retryable_read(error: &Error) -> bool {
	error.chain().any(|cause: &(dyn std::error::Error + 'static)| -> bool {
		match cause.downcast_ref::<ureq::Error>() {
			Some(ureq::Error::Timeout(_) | ureq::Error::ConnectionFailed) => true,
			Some(ureq::Error::Io(error) | ureq::Error::Decompress(_, error)) => matches!(error.kind(),
				ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted
				| ErrorKind::BrokenPipe | ErrorKind::TimedOut | ErrorKind::Interrupted),
			_ => false,
		}
	})
}

fn read_response(response: Response<Body>) -> Result<String, Error> {
	if response.status().is_success() {
		let mut body: Body = response.into_body();
		let mut reader: BodyReader<'_> = body.as_reader();
		let mut content: String = String::new();
		reader.read_to_string(&mut content).map_err(ureq::Error::from)?;
		Ok(content)
	} else {
		Err(anyhow!("{}", response.status()))
	}
}
