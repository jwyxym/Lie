mod api;
mod get;

use std::{
	path::PathBuf,
	sync::OnceLock
};
use tauri::{
	path::BaseDirectory,
	Manager
};

pub static PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            api::get_ani,
            api::get_browse,
            api::get_schedule,
            api::get_video
        ])
		.setup(|app| {
			#[cfg(target_os = "android")]
			{
				let path: PathBuf = app.path().resolve("./", BaseDirectory::Public)?;
                PATH.set(path).ok();
			}
			#[cfg(not(target_os = "android"))]
			{
				let path: PathBuf = app.path().resolve("./", BaseDirectory::Resource)?;
				PATH.set(path).ok();
			}
			Ok(())
		})
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
