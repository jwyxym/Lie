use serde::Serialize;

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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp4RequestMode {
	Play,
	Download,
}