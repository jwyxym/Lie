use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseResult {
	pub list: Vec<BrowseItem>,
	pub has_more: bool,
}

#[derive(Serialize)]
pub struct BrowseItem {
	pub name: String,
	pub url: String,
	pub img: String,
	pub desc: String,
}

#[derive(Serialize)]
pub struct Schedule {
    pub name: String,
    pub url: String,
    pub img: String,
    pub count: String,
}

#[derive(Serialize)]
pub struct Anthology {
    pub name: String,
    pub url: String,
}
