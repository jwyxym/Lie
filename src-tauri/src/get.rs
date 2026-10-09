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

use anyhow::{Error, Result, anyhow};
use std::{io::Read, sync::LazyLock};
use ureq::{Body, BodyReader, http::Response};

const NEXT_BASE_URL: &str = "https://lie.ygopro3.cn";
pub(super) static AGENT: LazyLock<ureq::Agent> = LazyLock::new(ureq::agent);

async fn get(url: &str) -> Result<String, Error> {
	let cache_key: String = format!("GET:{}", url);
	if let Some(content) = cache::get(cache_key.clone()) {
		return Ok(content);
	}
	let response: Response<Body> = AGENT.get(url).call()?;
	let content: String = read_response(response)?;
	cache::set(cache_key, content.clone());
	Ok(content)
}

fn read_response(response: Response<Body>) -> Result<String, Error> {
	if response.status().is_success() {
		let mut body: Body = response.into_body();
		let mut reader: BodyReader<'_> = body.as_reader();
		let mut content: String = String::new();
		reader.read_to_string(&mut content)?;
		Ok(content)
	} else {
		Err(anyhow!("{}", response.status()))
	}
}
