use super::{Anthology, NEXT_BASE_URL, get, next::read_props};
use anyhow::{Context, Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;

lazy_static! {
	static ref DESC_SEL: Selector = Selector::parse("p.whitespace-pre-line").unwrap();
}

#[derive(Deserialize)]
struct Details {
	anime: Anime,
	sources: Vec<Source>,
}

#[derive(Deserialize)]
struct Anime {
	id: u64,
	title: String,
	cover_url: Option<String>,
	release_date: Option<String>,
	release_year: Option<u32>,
}

#[derive(Deserialize)]
struct Source {
	code: String,
	episodes: Vec<Episode>,
}

#[derive(Deserialize)]
struct Episode {
	id: u64,
	kind: String,
	title: Option<String>,
	episode_number: Value,
}

pub async fn ani(
	url: String,
) -> Result<(String, String, String, String, Vec<Vec<Anthology>>), Error> {
	let html = get(&url).await?;
	let document = Html::parse_document(&html);
	let Details { anime, sources } = read_details(&document)?;
	if anime.title.trim().is_empty() {
		return Err(anyhow!("cannot find animation title"));
	}
	// Read the full synopsis from the DOM; JSON-LD contains a shortened version.
	let desc = document.select(&DESC_SEL).next()
		.map(|el| el.text().collect::<String>().trim().to_owned())
		.unwrap_or_default();
	let date = anime.release_date.filter(|date| !date.trim().is_empty())
		.or_else(|| anime.release_year.map(|year| year.to_string()))
		.unwrap_or_default();
	let img = anime.cover_url.unwrap_or_default();
	let mut anthologies = Vec::new();
	for source in sources {
		if source.code.trim().is_empty() {
			return Err(anyhow!("cannot find animation source code"));
		}
		let mut list = Vec::new();
		for episode in source.episodes {
			list.push(Anthology {
				name: episode_name(&episode)?,
				url: format!("{NEXT_BASE_URL}/anime/{}/play/{}?source={}",
					anime.id, episode.id, urlencoding::encode(&source.code)),
			});
		}
		anthologies.push(list);
	}
	Ok((anime.title, date, desc, img, anthologies))
}

fn read_details(document: &Html) -> Result<Details, Error> {
	let props = read_props(document, |value| {
		value.get("anime").is_some_and(Value::is_object)
			&& value.get("sources").is_some_and(Value::is_array)
	}).context("cannot find animation details in Next.js page data")?;
	serde_json::from_value(props).context("invalid animation details or episode data")
}

fn episode_name(episode: &Episode) -> Result<String, Error> {
	let number = episode.episode_number.as_f64()
		.or_else(|| episode.episode_number.as_str()?.parse::<f64>().ok())
		.filter(|number| number.is_finite())
		.ok_or_else(|| anyhow!("invalid episode number: {}", episode.id))?;
	let number = if number.fract() == 0.0 {
		format!("{number:02.0}")
	} else {
		number.to_string()
	};
	let kind = match episode.kind.as_str() {
		"main" => "正片",
		"sp" => "SP",
		"ova" => "OVA",
		"oad" => "OAD",
		"movie" => "剧场版",
		"ex" => "特典",
		kind => kind,
	};
	let mut name = if episode.kind == "main" {
		format!("第{number}集")
	} else {
		format!("{kind} {number}")
	};
	if let Some(title) = episode.title.as_deref().map(str::trim)
		.filter(|title| !title.is_empty() && !(title.starts_with('第') && title.ends_with('集')))
	{
		name.push_str(" · ");
		name.push_str(title);
	}
	Ok(name)
}
