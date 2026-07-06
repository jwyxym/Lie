use serde::Serialize;

#[derive(Serialize)]
pub struct Schedule {
	pub name: String,
	pub url: String,
	pub img: String,
	pub count: String
}