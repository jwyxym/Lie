use super::{get, Schedule};
use anyhow::Error;
use scraper::{Html, Selector};
use lazy_static::lazy_static;
use std::collections::BTreeMap;

const URL : &str = "https://next.xifanacg.com/schedule";

lazy_static! {
	static ref DAY_SEL: Vec<Selector> = {
		let mut vec: Vec<Selector> = Vec::new();
		for i in 0..7 {
			vec.push(Selector::parse(&format!(r#"div[id^="day-{}"] ul li a"#, i)).unwrap());
		}
		vec
	};
	static ref IMG_SEL: Selector = Selector::parse("img").unwrap();
	static ref DIV_SEL: Selector = Selector::parse("div > div").unwrap();
}

pub async fn schedule () -> Result<BTreeMap<usize, Vec<Schedule>>, Error> {
	let html: String = get(URL).await?;
	let document: Html = Html::parse_document(&html);
	let mut schedule: BTreeMap<usize, Vec<Schedule>> = BTreeMap::new();

	for i in 0..7 {
		let mut list: Vec<Schedule> = Vec::new();
		for a in document.select(&DAY_SEL[i]) {
			if let Some(href) = a.value().attr("href")
				&& let Some(name )= a.value().attr("aria-label")
				&& let Some(img) = a.select(&IMG_SEL).next()
				&& let Some(div) = a.select(&DIV_SEL).next()
			{
				let src: &str = img.value().attr("src").unwrap_or("");
				let ct: String = div.text().collect::<String>();
				list.push(Schedule {
					name: String::from(name),
					url: format!("https://next.xifanacg.com{}", href),
					img: format!("https://next.xifanacg.com{}", src),
					count: ct
				});
			}
		}
		schedule.insert(i, list);
	}
	Ok(schedule)
}