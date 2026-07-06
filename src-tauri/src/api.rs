use super::get::*;
use std::collections::BTreeMap;

#[tauri::command]
pub async fn get_all() -> String {
	all(2026, 0).await.unwrap()
}

#[tauri::command]
pub async fn get_schedule() -> BTreeMap<usize, Vec<Schedule>> {
	schedule().await.unwrap()
}
