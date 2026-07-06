use super::get;
use anyhow::{Error, anyhow};
use scraper::{Html, Selector};

const URL : &str = "https://next.xifanacg.com/browse";

pub async fn all (year: i64, page: i64) -> Result<String, Error> {
	let url: String = if year > 0 {
		format!("{}?year={}&page={}", URL, year, page)
	} else {
		format!("{}?page={}", URL, page)
	};
	let html: String = get(&url).await?;
	let document: Html = Html::parse_document(&html);

	let sel: Selector = Selector::parse(r#"div[id^="S:1"] > script"#)
		.map_err(|e |anyhow!(e.to_string()))?;
	document
		.select(&sel)
		.next()
		.map(|s| s.text().collect::<Vec<_>>().join(""))
		.ok_or(anyhow!("cannot find script"))
}