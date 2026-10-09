use super::{BrowseItem, BrowseResult, NEXT_BASE_URL, get};
use anyhow::{Context, Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use tauri::Url;

lazy_static! {
	static ref JSON_SEL: Selector = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
	static ref CARD_SEL: Selector = Selector::parse(r#"a[href^="/anime/"]"#).unwrap();
	static ref DESC_SEL: Selector = Selector::parse("p.text-xs").unwrap();
	static ref PAGE_SEL: Selector = Selector::parse(r#"nav[aria-label="分页"] a[href]"#).unwrap();
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemList {
	number_of_items: usize,
	item_list_element: Vec<ListEntry>,
}

#[derive(Deserialize)]
struct ListEntry {
	item: Anime,
}

#[derive(Deserialize)]
struct Anime {
	name: String,
	url: String,
	image: Option<String>,
	#[serde(rename = "datePublished")]
	date: Option<String>,
}

pub async fn browse(status: u8, year: i64, page: i64) -> Result<BrowseResult, Error> {
	let finished = match status {
		1 => "0",
		2 => "1",
		_ => return Err(anyhow!("invalid browse status: {status}")),
	};
	if year < -1 || page < 1 {
		return Err(anyhow!("invalid browse year or page"));
	}
	let mut url = Url::parse(&format!("{NEXT_BASE_URL}/browse"))?;
	{
		let mut query = url.query_pairs_mut();
		query.append_pair("finished", finished);
		if year == -1 {
			query.append_pair("older", "1");
		} else if year != 0 {
			query.append_pair("year", &year.to_string());
		}
		query.append_pair("page", &page.to_string());
	}
	let html = get(url.as_str()).await?;
	let document = Html::parse_document(&html);
	let mut items = None;
	for script in document.select(&JSON_SEL) {
		let value: Value = serde_json::from_str(&script.text().collect::<String>())
			.context("invalid browse structured data")?;
		if value["@type"] == "ItemList"
			&& value["@id"].as_str().and_then(|id| url.join(id).ok())
				.is_some_and(|id| id.path() == "/browse" && id.fragment() == Some("item-list"))
		{
			items = Some(serde_json::from_value::<ItemList>(value)
				.context("invalid browse item list")?);
			break;
		}
	}
	let items = items.ok_or_else(|| anyhow!("cannot find browse item list"))?;
	if items.number_of_items != items.item_list_element.len() {
		return Err(anyhow!("incomplete browse item list"));
	}
	let mut descriptions = BTreeMap::new();
	for card in document.select(&CARD_SEL) {
		if let Some(desc) = card.select(&DESC_SEL).next() {
			let href = card.value().attr("href").unwrap();
			descriptions.insert(url.join(href)?.to_string(), desc.text().collect::<String>());
		}
	}
	let mut list = Vec::new();
	for entry in items.item_list_element {
		let anime = entry.item;
		if anime.name.trim().is_empty() || anime.url.trim().is_empty() {
			return Err(anyhow!("browse item is missing a title or URL"));
		}
		// Structured data retains the original site's domain when served through a mirror.
		let mut item_url = url.join(&anime.url)?;
		item_url.set_scheme(url.scheme()).map_err(|_| anyhow!("invalid browse item URL scheme"))?;
		item_url.set_host(url.host_str())?;
		item_url.set_port(url.port()).map_err(|_| anyhow!("invalid browse item URL port"))?;
		let item_url = item_url.to_string();
		let desc = descriptions.remove(&item_url).or(anime.date).unwrap_or_default();
		list.push(BrowseItem {
			name: anime.name,
			url: item_url,
			img: anime.image.unwrap_or_default(),
			desc: desc.trim().to_owned(),
		});
	}
	let has_more = !list.is_empty() && document.select(&PAGE_SEL).any(|link| {
		let Some(href) = link.value().attr("href") else { return false };
		let Ok(next) = url.join(href) else { return false };
		next.query_pairs().any(|(name, value)| {
			name == "page" && value.parse::<i64>().is_ok_and(|next_page| next_page > page)
		})
	});
	Ok(BrowseResult { list, has_more })
}
