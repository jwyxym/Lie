use super::{Anthology, BASE_URL, get};
use anyhow::{Error, anyhow};
use lazy_static::lazy_static;
use scraper::{Html, Selector};

lazy_static! {
    static ref TITLE_SEL: Selector = Selector::parse(r#"h3[class^="slide-info-title"]"#).unwrap();
    static ref DESC_SEL: Selector = Selector::parse(r#"div[id^="height_limit"]"#).unwrap();
    static ref PIC_SEL: Selector = Selector::parse(r#"div[class^="detail-pic"] img"#).unwrap();
    static ref DATE_SEL: Selector = Selector::parse(r#"div[class^="detail-info"] a"#).unwrap();
    static ref UL_SEL: Selector = Selector::parse(r#"div[class^="anthology-list"] ul"#).unwrap();
    static ref A_SEL: Selector = Selector::parse("a").unwrap();
}

pub async fn ani(
    url: String,
) -> Result<(String, String, String, String, Vec<Vec<Anthology>>), Error> {
    let html: String = get(&url).await?;
    let document: Html = Html::parse_document(&html);

    let mut anthologys: Vec<Vec<Anthology>> = Vec::new();
    for ul in document.select(&UL_SEL) {
        let mut list: Vec<Anthology> = Vec::new();
        for a in ul.select(&A_SEL) {
            if let Some(href) = a.value().attr("href") {
                let name: String = a.text().collect::<String>();
                list.push(Anthology {
                    name: name,
                    url: format!("{}/{}", BASE_URL, href),
                });
            }
        }
        anthologys.push(list);
    }

    if let Some(img) = document.select(&PIC_SEL).next()
        && let Some(title) = document.select(&TITLE_SEL).next()
        && let Some(desc) = document.select(&DATE_SEL).next()
        && let Some(date) = document.select(&DESC_SEL).next()
    {
        if let Some(img) = img.value().attr("data-src") {
            let title: String = title.text().collect::<String>();
            let desc: String = desc.text().collect::<String>();
            let date: String = date.text().collect::<String>();
            return Ok((title, desc, date, img.to_string(), anthologys));
        }
    }
    Err(anyhow!("cannot find animation info"))
}
