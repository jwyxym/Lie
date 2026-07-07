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

async fn get_bytes(url: &str) -> Result<Vec<u8>, Error> {
	if let Some(content) = cache::get_bytes(url.to_string()) {
		return Ok(content);
	}
	let response: Response<Body> = ureq::get(url).call()?;
	let content: Vec<u8> = read_response_bytes(response)?;
	cache::set_bytes(url.to_string(), content.clone());
	Ok(content)
}

async fn mp4_request_mode(url: &str) -> Result<Mp4RequestMode, Error> {
	let response: Response<Body> = ureq::head(url)
		.header("Accept", "video/mp4,video/*,*/*")
		.header(
			"User-Agent",
			"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
			(KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
		)
		.call()?;

	if !response.status().is_success() {
		return Err(anyhow!("{}", response.status()));
	}

	let content_disposition = response_header(&response, "content-disposition")
		.unwrap_or("")
		.to_ascii_lowercase();
	if content_disposition.contains("attachment") {
		return Ok(Mp4RequestMode::Download);
	}

	let content_type = response_header(&response, "content-type")
		.unwrap_or("")
		.to_ascii_lowercase();
	if content_type.starts_with("video/") || content_type.contains("mp4") {
		return Ok(Mp4RequestMode::Play);
	}

	if url.split('?').next().unwrap_or("").ends_with(".mp4") {
		return Ok(Mp4RequestMode::Play);
	}

	Ok(Mp4RequestMode::Download)
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

fn response_header<'a>(response: &'a Response<Body>, name: &str) -> Option<&'a str> {
	response.headers().get(name)?.to_str().ok()
}

fn read_response_bytes(response: Response<Body>) -> Result<Vec<u8>, Error> {
	if response.status().is_success() {
		let mut body: Body = response.into_body();
		let mut reader: BodyReader<'_> = body.as_reader();
		let mut content: Vec<u8> = Vec::new();
		reader.read_to_end(&mut content)?;
		Ok(content)
	} else {
		Err(anyhow!("{}", response.status()))
	}
}