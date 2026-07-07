use super::{BASE_URL, Schedule, get};
use anyhow::Error;
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::collections::BTreeMap;

lazy_static! {
    static ref DAY_SEL: Vec<Selector> = {
        let mut vec: Vec<Selector> = Vec::new();
        for i in 1..8 {
            vec.push(
                Selector::parse(&format!(r#"div[id^="week-module-{}"] > div > div"#, i)).unwrap(),
            );
        }
        vec
    };
    static ref A_SEL: Selector = Selector::parse(r#"a[class^="time-title"]"#).unwrap();
    static ref IMG_SEL: Selector = Selector::parse("img").unwrap();
    static ref DIV_SEL: Selector =
        Selector::parse(r#"div[class^="public-list-subtitle"]"#).unwrap();
}

pub async fn schedule() -> Result<BTreeMap<usize, Vec<Schedule>>, Error> {
    let html: String = get(BASE_URL).await?;
    let document: Html = Html::parse_document(&html);
    let mut schedule: BTreeMap<usize, Vec<Schedule>> = BTreeMap::new();

    for i in 0..7 {
        let mut list: Vec<Schedule> = Vec::new();
        for el in document.select(&DAY_SEL[i]) {
            if let Some(img) = el.select(&IMG_SEL).next()
                && let Some(div) = el.select(&DIV_SEL).next()
                && let Some(a) = el.select(&A_SEL).next()
            {
                let img: &str = img.value().attr("data-src").unwrap_or("");
                let href: &str = a.value().attr("href").unwrap_or("");
                let name: String = a.text().collect::<String>();
                let ct: String = div.text().collect::<String>();
                list.push(Schedule {
                    name: String::from(name),
                    url: format!("{}{}", BASE_URL, href),
                    img: img.to_string(),
                    count: ct,
                });
            }
        }
        schedule.insert(i, list);
    }
    Ok(schedule)
}
