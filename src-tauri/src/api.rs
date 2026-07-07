use super::get::*;
use std::collections::BTreeMap;
use tauri::ipc::Response;

#[tauri::command]
pub async fn get_ani(
	url: String,
) -> Result<(String, String, String, String, Vec<Vec<Anthology>>), String> {
	ani(url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_browse(year: i64, page: i64) -> Result<String, String> {
	browse(year, page).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_schedule() -> Result<BTreeMap<usize, Vec<Schedule>>, String> {
	schedule().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_video(url: String) -> Response {
	video(url).await
		.ok()
		.map(Response::new)
		.unwrap_or_else(|| Response::new(Vec::new()))
}
