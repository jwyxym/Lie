use super::{NEXT_BASE_URL, Schedule, get};
use anyhow::{Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::collections::BTreeMap;

lazy_static! {
	// The new schedule page uses Monday = 0 through Sunday = 6.
	static ref DAY_SEL: Vec<Selector> = (0..7)
		.map(|day| Selector::parse(&format!("#day-{day}")).unwrap())
		.collect();
	static ref A_SEL: Selector = Selector::parse(r#"a[href^="/anime/"]"#).unwrap();
	static ref IMG_SEL: Selector = Selector::parse("img").unwrap();
	static ref COUNT_SEL: Selector = Selector::parse("img + div").unwrap();
}

pub async fn schedule() -> Result<BTreeMap<usize, Vec<Schedule>>, Error> {
	let html = get(&format!("{NEXT_BASE_URL}/schedule")).await?;
	let document = Html::parse_document(&html);
	let mut schedule = BTreeMap::new();

	for (day, selector) in DAY_SEL.iter().enumerate() {
		let section = document.select(selector).next()
			.ok_or_else(|| anyhow!("cannot find schedule day-{day} in {NEXT_BASE_URL}/schedule"))?;
		let mut list = Vec::new();
		for a in section.select(&A_SEL) {
			let href = a.value().attr("href").unwrap();
			let img = a.select(&IMG_SEL).next()
				.ok_or_else(|| anyhow!("cannot find schedule cover: {href}"))?;
			let name = a.value().attr("aria-label")
				.or_else(|| img.value().attr("alt"))
				.filter(|name| !name.trim().is_empty())
				.ok_or_else(|| anyhow!("cannot find schedule title: {href}"))?;
			let src = img.value().attr("src")
				.filter(|src| !src.trim().is_empty())
				.ok_or_else(|| anyhow!("cannot find schedule image source: {href}"))?;
			let count = a.select(&COUNT_SEL).next()
				.map(|el| el.text().collect::<String>().trim().to_owned())
				.unwrap_or_default();
			let img = if src.starts_with("//") {
				format!("https:{src}")
			} else if src.starts_with('/') {
				format!("{NEXT_BASE_URL}{src}")
			} else {
				src.to_owned()
			};
			list.push(Schedule {
				name: name.trim().to_owned(),
				url: format!("{NEXT_BASE_URL}{href}"),
				img,
				count,
			});
		}
		schedule.insert(day, list);
	}
	Ok(schedule)
}
