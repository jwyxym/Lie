mod anime;
mod browse;
mod cache;
mod schedule;
mod types;
mod video;
pub use anime::ani;
pub use browse::browse;
pub use schedule::schedule;
pub use types::*;
pub use video::video;

use anyhow::{Error, Result, anyhow};
use std::io::Read;
use ureq::{Body, BodyReader, http::Response};

const BASE_URL: &str = "https://anime.xifanacg.com";

async fn get(url: &str) -> Result<String, Error> {
	if let Some(content) = cache::get(url.to_string()) {
		return Ok(content);
	}
	let response: Response<Body> = ureq::get(url).call()?;
	let content: String = read_response(response)?;
	cache::set(url.to_string(), content.clone());
	Ok(content)
}

async fn post(url: &str, data: &str) -> Result<String, Error> {
	let cache_key = format!("POST:{}\n{}", url, data);
	if let Some(content) = cache::get(cache_key.clone()) {
		return Ok(content);
	}
	let response: Response<Body> = ureq::post(url)
		.header("Accept", "application/json, text/javascript, */*; q=0.01")
		.header(
			"Content-Type",
			"application/x-www-form-urlencoded; charset=UTF-8",
		)
		.header("X-Requested-With", "XMLHttpRequest")
		.send(data)?;

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