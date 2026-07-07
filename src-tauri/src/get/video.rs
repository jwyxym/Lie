use std::result;

use super::{get, get_bytes, mp4_request_mode, Mp4RequestMode};
use anyhow::{Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde_json::{Value, from_str};

lazy_static! {
	static ref SRC_SEL: Selector = Selector::parse("div.player-style-1 script").unwrap();
}
pub async fn video(url: String) -> Result<Vec<u8>, Error> {
	let html: String = get(&url).await?;
	let video_url: String = {
		let document: Html = Html::parse_document(&html);
		let src = document
			.select(&SRC_SEL)
			.next()
			.ok_or_else(|| anyhow!("cannot find video"))?;
		let src: String = src.text().collect::<String>();
		get_url(src)?
	};
	if video_url.contains(".mp4") {
		match mp4_request_mode(&video_url).await? {
			Mp4RequestMode::Play => {
				let mut result: Vec<u8> = vec![1];
				result.append(&mut video_url.as_bytes().to_vec());
				Ok(result)
			}
			Mp4RequestMode::Download => {
				let mut result: Vec<u8> = vec![2];
				result.append(&mut get_bytes(&video_url).await?);
				Ok(result)
			}
		}
	} else {
		let mut result: Vec<u8> = vec![1];
		result.append(&mut video_url.as_bytes().to_vec());
		Ok(result)
	}
}

fn get_url(script: String) -> Result<String, Error> {
	let json: &str = script
		.trim()
		.split_once("=")
		.map(|(_, json)| json)
		.ok_or_else(|| anyhow!("cannot find url"))?
		.trim()
		.trim_end_matches(';');

	let value: Value = from_str(json)?;
	value["url"]
		.as_str()
		.map(|url| url.to_string())
		.ok_or_else(|| anyhow!("cannot find url"))
}
