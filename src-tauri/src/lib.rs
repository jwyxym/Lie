mod api;
mod get;
mod download;
mod hls;
mod media;
mod database;

use std::{
	path::PathBuf,
	sync::{Arc, OnceLock}
};
use tauri::{App, Manager};

pub static PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
		.setup(|app: &mut App| {
			let downloads: download::Downloads = download::Downloads::initialize(app.handle())?;
			app.asset_protocol_scope().allow_directory(downloads.directory(), true)?;
			app.manage(Arc::new(downloads));
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			api::get_ani,
			api::get_browse,
			api::get_schedule,
			api::get_video,
			api::get_search,
			download::list_downloads,
			download::start_download,
			download::delete_download,
			download::cached_video,
			download::cached_video_bytes,
			download::cached_video_size,
			download::download_directory,
			download::get_watch_position,
			download::save_watch_position
		])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
