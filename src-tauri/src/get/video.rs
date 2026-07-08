use super::get;
use anyhow::{Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde_json::{Value, from_str};

lazy_static! {
	static ref SRC_SEL: Selector = Selector::parse("div.player-style-1 script").unwrap();
}
pub async fn video(url: String) -> Result<String, Error> {
	let html: String = get(&url).await?;
	let document: Html = Html::parse_document(&html);
	let src = document
		.select(&SRC_SEL)
		.next()
		.ok_or_else(|| anyhow!("cannot find video"))?;
	let src: String = src.text().collect::<String>();
	get_url(src)
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
