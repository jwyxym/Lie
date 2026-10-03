use anyhow::{Context, Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde_json::Value;

// Public publishable key shipped by the site's anonymous browser client.
pub(super) const PUBLISHABLE_KEY: &str = "sb_publishable_OBIVAWACIX6lPXrO98_z24_HcsmalkA";

lazy_static! {
	static ref SCRIPT_SEL: Selector = Selector::parse("script:not([src])").unwrap();
}

pub(super) fn read_props(document: &Html, matches: impl Fn(&Value) -> bool) -> Result<Value, Error> {
	// Flight records may span multiple script chunks. Decode JSON without
	// executing any JavaScript from the remote page.
	let mut stream = String::new();
	for script in document.select(&SCRIPT_SEL) {
		let text = script.text().collect::<String>();
		let Some(json) = text.trim().trim_end_matches(';')
			.strip_prefix("self.__next_f.push(")
			.and_then(|text| text.strip_suffix(')')) else {
			continue;
		};
		let chunk: Value = serde_json::from_str(json)
			.context("cannot parse Next.js page data chunk")?;
		if chunk[0].as_u64() == Some(1)
			&& let Some(text) = chunk[1].as_str()
		{
			stream.push_str(text);
		}
	}
	for line in stream.lines() {
		let Some((id, json)) = line.split_once(':') else { continue };
		if id.is_empty() || !id.bytes().all(|ch| ch.is_ascii_hexdigit()) {
			continue;
		}
		// Module, text and other Flight records are not JSON component trees.
		let Ok(value) = serde_json::from_str::<Value>(json) else { continue };
		if let Some(props) = find_props(&value, &matches) {
			return Ok(props.clone());
		}
	}
	Err(anyhow!("cannot find requested properties in Next.js page data"))
}

fn find_props<'a>(value: &'a Value, matches: &impl Fn(&Value) -> bool) -> Option<&'a Value> {
	if matches(value) {
		return Some(value);
	}
	match value {
		Value::Object(object) => object.values().find_map(|value| find_props(value, matches)),
		Value::Array(array) => array.iter().find_map(|value| find_props(value, matches)),
		_ => None,
	}
}
