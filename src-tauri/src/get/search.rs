use super::{AGENT, BrowseItem, BrowseResult, NEXT_BASE_URL, get, next::{PUBLISHABLE_KEY, read_props}};
use anyhow::{Context, Error, anyhow};
use scraper::Html;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;

const SEARCH_API: &str = "https://api.xifanacg.com/rest/v1/rpc/search_animes";
const PAGE_SIZE: u64 = 24;

#[derive(Deserialize)]
struct SearchItem {
	id: u64,
	title: String,
	cover_url: Option<String>,
	format: Option<String>,
	anime_type: Option<AnimeType>,
	release_date: Option<String>,
	release_year: Option<u32>,
	is_finished: Option<bool>,
	total_episodes: Option<u32>,
	current_episodes: Option<u32>,
	total_count: u64,
}

#[derive(Deserialize)]
struct AnimeType {
	name: String,
}

impl SearchItem {
	fn description(&self) -> String {
		let mut parts = Vec::new();
		let kind = match self.format.as_deref() {
			Some("movie") => Some("剧场版"),
			Some("tv") => Some("TV动画"),
			Some("web") => Some("网络动画"),
			Some("ova") => Some("OVA"),
			Some("oad") => Some("OAD"),
			Some("sp") => Some("特别篇"),
			_ => self.anime_type.as_ref().map(|kind| kind.name.as_str()),
		};
		if let Some(kind) = kind.filter(|kind| !kind.trim().is_empty()) {
			parts.push(kind.trim().to_owned());
		}
		if let Some(date) = self.release_date.as_deref().map(str::trim).filter(|date| !date.is_empty()) {
			parts.push(format!("{date}上映"));
		} else if let Some(year) = self.release_year.filter(|year| *year > 0) {
			parts.push(format!("{year}年"));
		}
		match self.is_finished {
			Some(true) => parts.push("已完结".to_owned()),
			Some(false) if self.current_episodes == Some(0) => parts.push("待更新".to_owned()),
			Some(false) => parts.push("连载中".to_owned()),
			None => {},
		}
		let total = self.total_episodes.filter(|count| *count > 0);
		let current = self.current_episodes.filter(|count| *count > 0);
		match (self.is_finished, current, total) {
			(Some(true), _, Some(total)) => parts.push(format!("全{total}集")),
			(_, Some(current), Some(total)) => parts.push(format!("更新至{current}/{total}集")),
			(_, Some(current), None) => parts.push(format!("更新至{current}集")),
			(_, None, Some(total)) => parts.push(format!("共{total}集")),
			_ => {},
		}
		parts.join(" · ")
	}
}

pub async fn search(keyword: String, page: u32) -> Result<BrowseResult, Error> {
	if page == 0 {
		return Err(anyhow!("invalid search page"));
	}
	let keyword = keyword.trim();
	if keyword.is_empty() {
		return Ok(BrowseResult { list: Vec::new(), has_more: false });
	}
	let url = format!("{NEXT_BASE_URL}/search?q={}", urlencoding::encode(keyword));
	let html = get(&url).await?;
	let document = Html::parse_document(&html);
	// Search results are loaded in the browser; the HTML only supplies the query.
	let props = read_props(&document, |value| value.get("initialQuery").is_some_and(Value::is_string))
		.context("cannot find search component in Next.js page data")?;
	if props["initialQuery"].as_str() != Some(keyword) {
		return Err(anyhow!("search page does not match requested keyword"));
	}
	let body = json!({
		"search_term": keyword,
		"page_number": page,
		"items_per_page": PAGE_SIZE,
		"sort_by": "created_at",
		"sort_order": "desc",
	}).to_string();
	let response = AGENT.post(SEARCH_API)
		.config().http_status_as_error(false)
		.timeout_global(Some(Duration::from_secs(20))).build()
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.header("apikey", PUBLISHABLE_KEY)
		.header("Authorization", &format!("Bearer {PUBLISHABLE_KEY}"))
		.header("Origin", NEXT_BASE_URL)
		.header("Referer", &url)
		.send(body.as_str())?;
	let status = response.status();
	let content = response.into_body().read_to_string()?;
	if !status.is_success() {
		let error = serde_json::from_str::<Value>(&content).unwrap_or_default();
		let message = error["message"].as_str().or(error["error"].as_str()).unwrap_or("search_failed");
		return Err(anyhow!("search request failed (HTTP {status}): {message}"));
	}
	let items: Vec<SearchItem> = serde_json::from_str(&content).context("invalid search results")?;
	let total = items.first().map_or(0, |item| item.total_count);
	let has_more = !items.is_empty() && u64::from(page) * PAGE_SIZE < total;
	let mut list = Vec::new();
	for item in items {
		if item.title.trim().is_empty() {
			return Err(anyhow!("search result is missing a title"));
		}
		let desc = item.description();
		list.push(BrowseItem {
			name: item.title,
			url: format!("{NEXT_BASE_URL}/anime/{}", item.id),
			img: item.cover_url.unwrap_or_default(),
			desc,
		});
	}
	Ok(BrowseResult { list, has_more })
}
