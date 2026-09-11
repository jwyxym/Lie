use super::{AGENT, BASE_URL, cache};
use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use ureq::{Body, http::Response};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::time::UNIX_EPOCH;

lazy_static! {
    static ref SEL: Selector = Selector::parse("div.row.mask2 > div").unwrap();
    static ref IMG_SEL: Selector = Selector::parse("img").unwrap();
    static ref A_SEL: Selector = Selector::parse("a").unwrap();
}

#[derive(Serialize)]
pub enum Verify {
    Image(VerifyImage),
    Data(Vec<(String, String, String)>)
}

#[derive(Serialize)]
pub struct VerifyImage {
    bytes: Vec<u8>,
    mime: String,
}

#[derive(Deserialize)]
struct VerifyResponse {
    code: i64,
    msg: String,
}

pub async fn image(keyword: String) -> Result<Verify, Error> {
	let cache_key: String = format!("VERIFY:{}", keyword);
    if let Some(html) = cache::get(cache_key) {
        return Ok(Verify::Data(search(keyword, html)))
    }
    let url: String = format!("{BASE_URL}/index.php/verify/index.html?r={}",
        UNIX_EPOCH.elapsed()?.as_millis());
    let response: Response<Body> = AGENT
        .get(&url)
        .header("Cache-Control", "no-cache")
        .call()?;

    if !response.status().is_success() {
        return Err(anyhow!("captcha request failed: {}", response.status()));
    }

    let mime: String = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/png")
        .to_string();
    let bytes = response.into_body().read_to_vec()?;
    Ok(Verify::Image(VerifyImage { bytes, mime }))
}

pub async fn submit(verify: String, keyword: String) -> Result<Vec<(String, String, String)>, Error> {
    let verify: &str = verify.trim();
    if verify.is_empty() {
        return Err(anyhow!("captcha cannot be empty"));
    }
    let keyword: &str = keyword.trim();
    if keyword.is_empty() {
        return Err(anyhow!("search keyword cannot be empty"));
    }

    let url: String = format!(
        "{BASE_URL}/index.php/ajax/verify_check?type=search&verify={}",
        urlencoding::encode(verify),
    );
    let response: Response<Body> = AGENT
        .post(&url)
        .header("Accept", "application/json, text/javascript, */*; q=0.01")
        .header("X-Requested-With", "XMLHttpRequest")
        .send_empty()?;

    if !response.status().is_success() {
        return Err(anyhow!("captcha submission failed: {}", response.status()));
    }

    let content: String = response.into_body().read_to_string()?;
    let result: VerifyResponse = serde_json::from_str(&content)?;
    if result.code == 1 {
        let search_url: String = format!(
            "{BASE_URL}/search/wd/{}.html",
            urlencoding::encode(keyword),
        );
        let response: Response<Body> = AGENT.get(&search_url).call()?;
        if !response.status().is_success() {
            return Err(anyhow!("search request failed: {}", response.status()));
        }
        let html: String = response.into_body().read_to_string()?;
        Ok(search(keyword.to_owned(), html))
    } else {
        Err(anyhow!("{}", result.msg))
    }
}

fn search (keyword: String, html: String) -> Vec<(String, String, String)> {
    let document: Html = Html::parse_document(&html);
	let cache_key: String = format!("VERIFY:{}", keyword);
    cache::set( cache_key, html);
    let mut results: Vec<(String, String, String)> = Vec::new();
    for div in document.select(&SEL) {
        if let Some(a) = div.select(&A_SEL).next() {
            if let Some(href) = a.value().attr("href") {
                let name: String = a.text().collect::<String>();
                if let Some(img) = div.select(&IMG_SEL).next() {
                    if let Some(img) = img.value().attr("data-src") {
                        results.push((name, href.to_owned(), img.to_owned()))
                    }
                }
            }
        }
    }
    results
}