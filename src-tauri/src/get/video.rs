use super::{AGENT, NEXT_BASE_URL, next::{PUBLISHABLE_KEY, read_props}, read_response};
use anyhow::{Context, Error, anyhow};
use scraper::Html;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;
use tauri::Url;

const PLAYBACK_API: &str = "https://api.xifanacg.com/functions/v1/issue-web-playback";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Player {
	anime_id: u64,
	episode_id: u64,
	initial_source_id: Option<u64>,
	sources: Vec<Source>,
}

#[derive(Deserialize)]
struct Source {
	id: u64,
	code: String,
	episodes: Vec<Episode>,
}

#[derive(Deserialize)]
struct Episode {
	id: u64,
}

pub async fn video(url: String) -> Result<String, Error> {
	let page_url = Url::parse(&url).context("invalid video page URL")?;
	let path: Vec<_> = page_url.path().trim_matches('/').split('/').collect();
	if page_url.origin().ascii_serialization() != NEXT_BASE_URL
		|| path.len() != 4 || path[0] != "anime" || path[2] != "play"
	{
		return Err(anyhow!("expected {NEXT_BASE_URL}/anime/<id>/play/<episode>"));
	}
	let anime_id = path[1].parse::<u64>().context("invalid anime ID")?;
	let episode_id = path[3].parse::<u64>().context("invalid episode ID")?;
	let response = AGENT.get(page_url.as_str())
		.config().timeout_global(Some(Duration::from_secs(30))).build()
		.call()?;
	let html = read_response(response)?;
	let document = Html::parse_document(&html);
	let props = read_props(&document, |value| {
		value.get("episodeId").is_some()
			&& value.get("initialSourceId").is_some()
			&& value.get("sources").is_some_and(Value::is_array)
	}).context("cannot find video player in Next.js page data")?;
	let player: Player = serde_json::from_value(props).context("invalid video player data")?;
	if player.anime_id != anime_id || player.episode_id != episode_id {
		return Err(anyhow!("video page does not match requested episode"));
	}
	let requested_source = page_url.query_pairs()
		.find(|(name, _)| name == "source").map(|(_, code)| code.into_owned());
	let source = player.sources.iter().find(|source| {
		requested_source.as_ref().map_or(
			Some(source.id) == player.initial_source_id,
			|code| source.code == *code,
		)
	}).ok_or_else(|| anyhow!("requested video source is unavailable"))?;
	if !source.episodes.iter().any(|episode| episode.id == episode_id) {
		return Err(anyhow!("episode is unavailable on source {}", source.code));
	}

	// Signed playback URLs expire. Do not use the shared permanent GET/POST cache.
	let body = json!({
		"action": "fallback",
		"episode_id": episode_id,
		"source_id": source.id,
	}).to_string();
	let response = AGENT.post(PLAYBACK_API)
		.config()
		.http_status_as_error(false)
		.timeout_global(Some(Duration::from_secs(20)))
		.build()
		.header("Content-Type", "application/json")
		.header("Accept", "application/json")
		.header("apikey", PUBLISHABLE_KEY)
		.header("Authorization", &format!("Bearer {PUBLISHABLE_KEY}"))
		.header("Origin", NEXT_BASE_URL)
		.header("Referer", page_url.as_str())
		.send(body.as_str())?;
	let status = response.status();
	let content = response.into_body().read_to_string()?;
	let result: Value = serde_json::from_str(&content)
		.map_err(|_| anyhow!("invalid playback API response (HTTP {status})"))?;
	if !status.is_success() || result["ok"].as_bool() != Some(true) {
		let error = result["error"].as_str().unwrap_or("unknown_error");
		let retry = result.get("retry_after")
			.map(|value| format!(", retry_after={value}"))
			.unwrap_or_default();
		return Err(anyhow!("playback request failed (HTTP {status}): {error}{retry}"));
	}
	if result["episode_id"].as_u64() != Some(episode_id)
		|| result["anime_id"].as_u64() != Some(anime_id)
	{
		return Err(anyhow!("playback response does not match requested episode"));
	}
	let src = if let Some(candidates) = result["candidates"].as_array() {
		candidates.iter().find(|candidate| candidate["source_id"].as_u64() == Some(source.id))
			.and_then(|candidate| candidate["url"].as_str())
	} else {
		result["url"].as_str()
	}.filter(|url| !url.trim().is_empty())
		.ok_or_else(|| anyhow!("cannot find video URL for source {}", source.code))?;
	let video_url = Url::parse(src).context("invalid playback URL")?;
	if !matches!(video_url.scheme(), "http" | "https") {
		return Err(anyhow!("unsupported playback URL scheme"));
	}
	Ok(video_url.into())
}
