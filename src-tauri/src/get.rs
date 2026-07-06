mod all;
mod schedule;
mod types;
mod cache;
pub use all::all;
pub use schedule::schedule;
pub use types::*;

use anyhow::{Result, Error, anyhow};
use std::io::Read;
use ureq::{
	http::Response,
	Body,
	BodyReader
};

async fn get (url: &str) -> Result<String, Error> {
	if let Some(content) = cache::get(url.to_string()) {
		return Ok(content);
	}
	let response: Response<Body> = ureq::get(url).call()?;
	if response.status().is_success() {
		let mut body: Body = response.into_body();
		let mut reader: BodyReader<'_> = body.as_reader();
		let mut content: String = String::new();
		reader.read_to_string(&mut content)?;
		cache::set(url.to_string(), content.clone());
		Ok(content)
	} else {
		Err(anyhow!("{}", response.status()))
	}
}