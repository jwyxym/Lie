use super::{BASE_URL, post};
use anyhow::Error;

pub async fn browse(year: i64, page: i64) -> Result<String, Error> {
    let url: String = format!("{}/index.php/ds_api/vod", BASE_URL);
    let data: String = format!(
        "type=2&class=&area=&year={}&lang=&version=&state=&letter=&time=&level=0&weekday=&by=time&page={}",
        if year == 0 {
            String::new()
        } else {
            year.to_string()
        },
        page
    );
    post(&url, &data).await
}
