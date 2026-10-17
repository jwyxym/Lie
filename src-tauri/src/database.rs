use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OpenFlags, Row, backup::{Backup, StepResult}};
use std::{fs, path::{Path, PathBuf}, time::Duration};
use tauri::{AppHandle, Manager, path::BaseDirectory};

const FILE_NAME: &str = "video-cache.sqlite3";

pub fn open(app: &AppHandle) -> Result<Connection> {
	let legacy: Option<PathBuf> = app.path().app_data_dir().ok().map(|directory: PathBuf| directory.join(FILE_NAME));
	#[cfg(target_os = "android")]
	{
		let path: PathBuf = app.path().resolve("./", BaseDirectory::Public)?;
		let directory: PathBuf = path.parent().context("Public 目录没有父目录")?.to_path_buf();
		open_at(&directory, &legacy.into_iter().collect::<Vec<PathBuf>>())
	}
	#[cfg(not(target_os = "android"))]
	{
		let resource: Result<PathBuf> = app.path().resolve("./", BaseDirectory::Resource).map_err(Into::into);
		let local: Result<PathBuf> = app.path().resolve("./", BaseDirectory::AppLocalData).map_err(Into::into);
		let mut sources: Vec<PathBuf> = Vec::new();
		if let Ok(directory) = &resource { sources.push(directory.join(FILE_NAME)); }
		if let Ok(directory) = &local { sources.push(directory.join(FILE_NAME)); }
		if let Some(legacy) = legacy { sources.push(legacy); }
		let preferred: Result<Connection> = resource.and_then(|directory: PathBuf| open_at(&directory, &sources));
		match preferred {
			Ok(connection) => Ok(connection),
			Err(error) => open_at(&local?, &sources)
				.with_context(|| format!("Resource 数据库无法写入（{error:#}），AppLocalData 回退也失败")),
		}
	}
}

fn has_data(connection: &Connection) -> Result<bool> {
	let tables: i64 = connection.query_row(
		"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('downloads','watch_positions')",
		[], |row: &Row<'_>| row.get(0),
	)?;
	Ok(tables > 0)
}

fn open_at(directory: &Path, sources: &[PathBuf]) -> Result<Connection> {
	fs::create_dir_all(directory).with_context(|| format!("无法创建数据库目录 {}", directory.display()))?;
	let path: PathBuf = directory.join(FILE_NAME);
	let mut connection: Connection = Connection::open(&path)?;
	connection.busy_timeout(Duration::from_secs(5))?;
	// Check the database and its journal are writable, rather than just checking the folder.
	connection.execute_batch("PRAGMA journal_mode=WAL; BEGIN IMMEDIATE; COMMIT;")?;
	if !has_data(&connection)? {
		let destination: PathBuf = path.canonicalize()?;
		for source in sources {
			if !source.is_file() || source.canonicalize()? == destination { continue; }
			let original: Connection = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
			original.busy_timeout(Duration::from_secs(5))?;
			if !has_data(&original)? { continue; }
			// SQLite backup includes committed WAL data; copying only the .sqlite3 file does not.
			{
				let backup: Backup<'_, '_> = Backup::new(&original, &mut connection)?;
				if !matches!(backup.step(-1)?, StepResult::Done) { bail!("旧数据库正在使用中，请关闭其他应用实例后重试"); }
			}
			break;
		}
	}
	connection.execute_batch("PRAGMA journal_mode=WAL; BEGIN IMMEDIATE; COMMIT;")?;
	Ok(connection)
}
