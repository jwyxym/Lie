use super::get::*;
use std::collections::BTreeMap;

#[tauri::command]
pub async fn get_ani(
	url: String,
) -> Result<(String, String, String, String, Vec<Vec<Anthology>>), String> {
	ani(url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_browse(status: u8, year: i64, page: i64) -> Result<String, String> {
	browse(status, year, page).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_schedule() -> Result<BTreeMap<usize, Vec<Schedule>>, String> {
	schedule().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_video(url: String) -> Result<String, String> {
	video(url).await.map_err(|e| e.to_string())
}