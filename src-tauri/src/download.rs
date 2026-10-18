use anyhow::{Context, Result, anyhow, bail};
use crate::hls::{CancelToken, Progress};
use reqwest::{Client, Response};
use rusqlite::{Connection, OptionalExtension, Row, Statement, params};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs,
	future::Future,
	io::{Read, Seek, SeekFrom},
	path::{Path, PathBuf},
	pin::Pin,
	sync::{Arc, Mutex, MutexGuard},
	time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, State, Url, Wry, async_runtime::JoinHandle, ipc::Response as IpcResponse, plugin::{Builder as PluginBuilder, TauriPlugin}};
use tokio::{fs::File, io::AsyncWriteExt, sync::{OwnedSemaphorePermit, Semaphore, watch}};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
	id: i64,
	anime_id: String,
	episode_id: String,
	anime_title: String,
	episode_title: String,
	cover_url: String,
	source: String,
	page_url: String,
	status: String,
	downloaded_bytes: i64,
	total_bytes: Option<i64>,
	completed_segments: i64,
	total_segments: i64,
	merge_progress: Option<u8>,
	error: String,
}

impl Download {
	#[cfg(target_os = "android")]
	pub(crate) fn is_active(&self) -> bool {
		matches!(self.status.as_str(), "queued" | "downloading" | "merging")
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
	pub page_url: String,
	pub anime_title: String,
	pub episode_title: String,
	#[serde(default)]
	pub cover_url: String,
}

#[derive(Debug)]
struct Job {
	cancel: watch::Sender<bool>,
	finished: watch::Sender<bool>,
	cancel_reason: Mutex<Option<String>>,
}

impl CancelToken for Job {
	fn is_cancelled(&self) -> bool {
		*self.cancel.borrow()
	}

	fn cancelled(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
		Box::pin(async move {
			let mut receiver: watch::Receiver<bool> = self.cancel.subscribe();
			while !*receiver.borrow_and_update() {
				if receiver.changed().await.is_err() {
					break;
				}
			}
		})
	}
}

pub struct Downloads {
	db: Mutex<Connection>,
	jobs: Mutex<HashMap<i64, Arc<Job>>>,
	directory: PathBuf,
	slots: Arc<Semaphore>,
}

const COLUMNS: &str = "id, anime_id, episode_id, anime_title, episode_title, source, page_url, status, downloaded_bytes, total_bytes, completed_segments, total_segments, error, cover_url, merge_progress";

#[cfg(target_os = "android")]
mod android_background {
	use super::*;
	use anyhow::Context;
	use serde::{Deserialize, Serialize};
	use serde_json::Value;
	use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};
	use tauri::{Manager, ipc::{Channel, InvokeResponseBody}, plugin::{PluginApi, PluginHandle}};

	struct Background {
		handle: PluginHandle<Wry>,
		events: Channel<Value>,
		last_prepare: Mutex<Option<Instant>>,
	}

	#[derive(Deserialize)]
	struct NativeEvent {
		action: String,
		#[serde(default)]
		id: i64,
	}

	#[derive(Serialize)]
	struct Preparation<'a> {
		events: &'a Channel<Value>,
	}

	#[derive(Serialize)]
	#[serde(rename_all = "camelCase")]
	struct Snapshot {
		tasks: Vec<Download>,
		started_id: Option<i64>,
	}

	pub fn initialize(app: &AppHandle, api: PluginApi<Wry, ()>) -> Result<()> {
		let handle: PluginHandle<Wry> = api.register_android_plugin("cn.jwyxym.lie", "DownloadPlugin")?;
		let callback_app: AppHandle = app.clone();
		let events: Channel<Value> = Channel::new(move |body: InvokeResponseBody| {
			let event: NativeEvent = body.deserialize()?;
			if let Some(downloads) = callback_app.try_state::<Arc<Downloads>>() {
				match event.action.as_str() {
					"cancel" => { let _ = downloads.cancel(event.id, "下载已取消"); }
					"timeout" => { let _ = downloads.cancel_all("后台下载达到系统时限，请返回应用重试"); }
					"stopped" => { let _ = downloads.cancel_all("后台下载服务已停止，请重试"); }
					_ => {}
				}
			}
			Ok(())
		});
		app.manage(Background { handle, events, last_prepare: Mutex::new(None) });
		let monitor_app: AppHandle = app.clone();
		tauri::async_runtime::spawn(async move {
			loop {
				tokio::time::sleep(Duration::from_secs(1)).await;
				let background: State<'_, Background> = monitor_app.state::<Background>();
				let prepared: Option<Instant> = background.last_prepare.lock().ok().and_then(|value| *value);
				let Some(prepared) = prepared else { continue; };
				let Some(downloads) = monitor_app.try_state::<Arc<Downloads>>() else { continue; };
				let Ok(tasks) = downloads.list() else { continue; };
				let active: bool = tasks.iter().any(Download::is_active);
				let result: Result<(), _> = background.handle.run_mobile_plugin_async("update", Snapshot { tasks, started_id: None }).await;
				if let Err(error) = result {
					let _ = downloads.cancel_all(&format!("无法更新后台下载服务：{error}"));
				}
				if !active && prepared.elapsed() >= Duration::from_secs(5) {
					if let Ok(mut latest) = background.last_prepare.lock() {
						if *latest == Some(prepared) { *latest = None; }
					}
				}
			}
		});
		Ok(())
	}

	pub async fn prepare(app: &AppHandle) -> Result<()> {
		let background: State<'_, Background> = app.state::<Background>();
		let _: Value = background.handle.run_mobile_plugin_async("prepare", Preparation { events: &background.events })
			.await.context("无法启动安卓后台下载服务")?;
		*background.last_prepare.lock().map_err(|_| anyhow::anyhow!("后台下载服务状态不可用"))? = Some(Instant::now());
		Ok(())
	}

	pub async fn take_open(app: &AppHandle) -> Result<bool> {
		let background: State<'_, Background> = app.state::<Background>();
		background.handle.run_mobile_plugin_async("takeOpen", ()).await.map_err(Into::into)
	}

	pub async fn track(app: &AppHandle, id: i64) -> Result<()> {
		let tasks: Vec<Download> = app.state::<Arc<Downloads>>().list()?;
		app.state::<Background>().handle.run_mobile_plugin_async("update", Snapshot { tasks, started_id: Some(id) })
			.await.map_err(Into::into)
	}
}

pub fn background_plugin() -> TauriPlugin<Wry> {
	PluginBuilder::new("download-background")
		.setup(|_app: &AppHandle, _api| {
			#[cfg(target_os = "android")]
			android_background::initialize(_app, _api)?;
			Ok(())
		})
		.build()
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
	mutex.lock().map_err(|_| anyhow!("缓存状态锁不可用，请重启应用"))
}

fn read_download(row: &Row<'_>) -> rusqlite::Result<Download> {
	Ok(Download {
		id: row.get(0)?, anime_id: row.get(1)?, episode_id: row.get(2)?,
		anime_title: row.get(3)?, episode_title: row.get(4)?, source: row.get(5)?,
		page_url: row.get(6)?, status: row.get(7)?, downloaded_bytes: row.get(8)?,
		total_bytes: row.get(9)?, completed_segments: row.get(10)?,
		total_segments: row.get(11)?, error: row.get(12)?, cover_url: row.get(13)?,
		merge_progress: row.get(14)?,
	})
}

impl Downloads {
	pub fn directory(&self) -> &Path {
		&self.directory
	}

	pub fn initialize(app: &AppHandle) -> Result<Self> {
		let directory: PathBuf = app.path().video_dir()?.join("Lie");
		fs::create_dir_all(&directory).context("无法创建视频缓存目录")?;
		let directory: PathBuf = directory.canonicalize()?;
		let db: Connection = crate::database::open(app)?;
		db.execute_batch("PRAGMA journal_mode=WAL;
			CREATE TABLE IF NOT EXISTS downloads (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				anime_id TEXT NOT NULL, episode_id TEXT NOT NULL,
				anime_title TEXT NOT NULL, episode_title TEXT NOT NULL,
				cover_url TEXT NOT NULL DEFAULT '',
				source TEXT NOT NULL, page_url TEXT NOT NULL,
				status TEXT NOT NULL DEFAULT 'queued',
				downloaded_bytes INTEGER NOT NULL DEFAULT 0, total_bytes INTEGER,
				completed_segments INTEGER NOT NULL DEFAULT 0,
				total_segments INTEGER NOT NULL DEFAULT 0,
				merge_progress INTEGER,
				error TEXT NOT NULL DEFAULT '',
				UNIQUE(anime_id, episode_id)
			);
			CREATE TABLE IF NOT EXISTS watch_positions (
				anime_id TEXT NOT NULL, episode_id TEXT NOT NULL,
				position_seconds REAL NOT NULL, duration_seconds REAL NOT NULL,
				updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
				PRIMARY KEY (anime_id, episode_id)
			);
			UPDATE downloads SET status='failed', error='下载被中断，请重试'
			WHERE status IN ('queued', 'downloading', 'merging');")?;
		let column_names: Vec<String> = {
			let mut columns: Statement<'_> = db.prepare("PRAGMA table_info(downloads)")?;
			let names: Vec<String> = columns.query_map([], |row: &Row<'_>| row.get(1))?.collect::<rusqlite::Result<_>>()?;
			names
		};
		if !column_names.iter().any(|name: &String| name == "cover_url") {
			db.execute_batch("ALTER TABLE downloads ADD COLUMN cover_url TEXT NOT NULL DEFAULT '';")?;
		}
		if !column_names.iter().any(|name: &String| name == "merge_progress") {
			db.execute_batch("ALTER TABLE downloads ADD COLUMN merge_progress INTEGER;")?;
		}
		let manager: Self = Self {
			db: Mutex::new(db), jobs: Mutex::new(HashMap::new()), directory,
			slots: Arc::new(Semaphore::new(2)),
		};
		for item in manager.list()? {
			if item.status == "deleting" {
				manager.remove_files(item.id)?;
				lock(&manager.db)?.execute("DELETE FROM downloads WHERE id=?1", [item.id])?;
			} else if item.status != "completed" {
				manager.remove_files(item.id)?;
			} else if !manager.file(item.id).is_file() {
				manager.fail(item.id, "缓存文件已丢失，请重试")?;
			}
		}
		Ok(manager)
	}

	fn file(&self, id: i64) -> PathBuf {
		self.directory.join(id.to_string()).join("video.mp4")
	}

	fn completed_file(&self, page_url: &str) -> Result<PathBuf> {
		let item: Download = self.find(page_url)?.context("本集尚未缓存")?;
		if item.status != "completed" { bail!("本集尚未完成缓存"); }
		Ok(self.file(item.id))
	}

	fn remove_files(&self, id: i64) -> Result<()> {
		if id <= 0 { bail!("无效缓存 ID"); }
		let path: PathBuf = self.directory.join(id.to_string());
		if path.exists() {
			let resolved: PathBuf = path.canonicalize()?;
			if resolved.parent() != Some(self.directory.as_path()) {
				bail!("缓存文件不在视频目录中");
			}
			fs::remove_dir_all(&resolved).context("无法删除缓存文件，请先停止播放后重试")?;
		}
		Ok(())
	}

	pub(crate) fn list(&self) -> Result<Vec<Download>> {
		let db: MutexGuard<'_, Connection> = lock(&self.db)?;
		let mut statement: Statement<'_> = db.prepare(&format!("SELECT {COLUMNS} FROM downloads ORDER BY anime_id, CAST(episode_id AS INTEGER), id"))?;
		let rows: Vec<Download> = statement.query_map([], read_download)?.collect::<rusqlite::Result<_>>()?;
		Ok(rows)
	}

	#[cfg(target_os = "android")]
	pub(crate) fn cancel(&self, id: i64, reason: &str) -> Result<()> {
		if let Some(job) = lock(&self.jobs)?.get(&id) {
			*lock(&job.cancel_reason)? = Some(reason.to_owned());
			job.cancel.send_replace(true);
		}
		Ok(())
	}

	#[cfg(target_os = "android")]
	pub(crate) fn cancel_all(&self, reason: &str) -> Result<()> {
		for job in lock(&self.jobs)?.values() {
			*lock(&job.cancel_reason)? = Some(reason.to_owned());
			job.cancel.send_replace(true);
		}
		Ok(())
	}

	fn fail(&self, id: i64, message: &str) -> Result<()> {
		lock(&self.db)?.execute("UPDATE downloads SET status='failed', error=?2 WHERE id=?1 AND status!='deleting'", params![id, message])?;
		Ok(())
	}

	fn progress(&self, id: i64, bytes: u64, total: Option<u64>, segments: usize, count: usize, status: &str, merge_progress: Option<u8>) -> Result<()> {
		let bytes: i64 = bytes.try_into()?;
		let total: Option<i64> = total.map(i64::try_from).transpose()?;
		let segments: i64 = segments.try_into()?;
		let count: i64 = count.try_into()?;
		lock(&self.db)?.execute("UPDATE downloads SET downloaded_bytes=MAX(downloaded_bytes,?2), total_bytes=?3, completed_segments=?4, total_segments=?5, status=?6, merge_progress=?7 WHERE id=?1 AND status!='deleting'",
			params![id, bytes, total, segments, count, status, merge_progress])?;
		Ok(())
	}

	fn find(&self, page_url: &str) -> Result<Option<Download>> {
		let (anime, episode, _): (String, String, String) = identity(page_url)?;
		let db: MutexGuard<'_, Connection> = lock(&self.db)?;
		Ok(db.query_row(&format!("SELECT {COLUMNS} FROM downloads WHERE anime_id=?1 AND episode_id=?2"), params![anime, episode], read_download).optional()?)
	}

	fn start(self: &Arc<Self>, request: DownloadRequest) -> Result<i64> {
		let (anime, episode, source): (String, String, String) = identity(&request.page_url)?;
		if request.anime_title.trim().is_empty() || request.episode_title.trim().is_empty() {
			bail!("番剧名和集名不能为空");
		}
		let mut jobs: MutexGuard<'_, HashMap<i64, Arc<Job>>> = lock(&self.jobs)?;
		if !request.cover_url.is_empty() {
			lock(&self.db)?.execute("UPDATE downloads SET cover_url=?2 WHERE anime_id=?1", params![anime, request.cover_url])?;
		}
		if let Some(existing) = self.find(&request.page_url)? {
			if existing.status == "deleting" { bail!("该集正在删除，请稍后再试"); }
			if jobs.contains_key(&existing.id) || (existing.status == "completed" && self.file(existing.id).is_file()) {
				return Ok(existing.id);
			}
			self.remove_files(existing.id)?;
		}
		let id: i64 = {
			let db: MutexGuard<'_, Connection> = lock(&self.db)?;
			db.execute("INSERT INTO downloads (anime_id, episode_id, anime_title, episode_title, source, page_url, cover_url) VALUES (?1,?2,?3,?4,?5,?6,?7)
				ON CONFLICT(anime_id, episode_id) DO UPDATE SET anime_title=excluded.anime_title, episode_title=excluded.episode_title,
				source=excluded.source, page_url=excluded.page_url, cover_url=CASE WHEN excluded.cover_url='' THEN downloads.cover_url ELSE excluded.cover_url END,
				status='queued', error='', downloaded_bytes=0, total_bytes=NULL, completed_segments=0, total_segments=0, merge_progress=NULL",
				params![anime, episode, request.anime_title, request.episode_title, source, request.page_url, request.cover_url])?;
			db.query_row("SELECT id FROM downloads WHERE anime_id=?1 AND episode_id=?2", params![anime, episode], |row: &Row<'_>| row.get(0))?
		};
		let (cancel, _): (watch::Sender<bool>, watch::Receiver<bool>) = watch::channel(false);
		let (finished, _): (watch::Sender<bool>, watch::Receiver<bool>) = watch::channel(false);
		let job: Arc<Job> = Arc::new(Job { cancel, finished, cancel_reason: Mutex::new(None) });
		jobs.insert(id, job.clone());
		let manager: Arc<Self> = self.clone();
		tauri::async_runtime::spawn(async move {
			let outcome: Result<()> = manager.run(id, request.page_url, job.clone()).await;
			if let Err(error) = outcome {
				let error: String = lock(&job.cancel_reason).ok().and_then(|reason| reason.clone()).unwrap_or_else(|| format!("{error:#}"));
				let cleanup: Result<()> = manager.remove_files(id);
				let message: String = match cleanup {
					Ok(()) => format!("{error:#}"),
					Err(cleanup_error) => format!("{error:#}；清理失败：{cleanup_error:#}"),
				};
				let _ = manager.fail(id, &message);
			}
			if let Ok(mut jobs) = lock(&manager.jobs) { jobs.remove(&id); }
			job.finished.send_replace(true);
		});
		Ok(id)
	}

	async fn run(self: &Arc<Self>, id: i64, page_url: String, job: Arc<Job>) -> Result<()> {
		let _permit: OwnedSemaphorePermit = tokio::select! {
			_ = job.cancelled() => bail!("下载已取消"),
			permit = self.slots.clone().acquire_owned() => permit?,
		};
		self.progress(id, 0, None, 0, 0, "downloading", None)?;
		let resolver: JoinHandle<Result<String>> = tauri::async_runtime::spawn_blocking(move || {
			tauri::async_runtime::block_on(crate::get::video(page_url))
		});
		let url: String = tokio::select! {
			_ = job.cancelled() => bail!("下载已取消"),
			result = resolver => result??,
		};
		if job.is_cancelled() { bail!("下载已取消"); }
		let output: PathBuf = self.file(id);
		let parent: &Path = output.parent().context("无效视频目录")?;
		tokio::fs::create_dir_all(parent).await?;
		let temporary: PathBuf = parent.join("download.mp4");
		let client: Client = crate::media::client()?;
		let media_url: Url = Url::parse(&url)?;
		let response: Response = tokio::select! {
			_ = job.cancelled() => bail!("下载已取消"),
			response = crate::media::get(&client, &media_url, None) => response?,
		};
		let content_type: String = response.headers().get("content-type").and_then(|value| value.to_str().ok()).unwrap_or_default().to_ascii_lowercase();
		let hls: bool = Url::parse(&url)?.path().to_ascii_lowercase().ends_with(".m3u8")
			|| response.url().path().to_ascii_lowercase().ends_with(".m3u8") || content_type.contains("mpegurl");
		if hls {
			let manager: Arc<Self> = self.clone();
			let progress: Arc<dyn Fn(Progress) + Send + Sync> = Arc::new(move |event: Progress| {
				let status: &str = if event.completed_segments == event.total_segments { "merging" } else { "downloading" };
				let _ = manager.progress(id, event.downloaded_bytes, None, event.completed_segments, event.total_segments, status, event.merge_progress);
			});
			let outcome: Result<()> = tokio::select! {
				_ = job.cancelled() => bail!("下载已取消"),
				result = crate::hls::download(client, response, &temporary, progress) => result,
			};
			outcome.context("m3u8 下载或合并失败")?;
		} else {
			self.download_mp4(id, response, &temporary, &job).await?;
		}
		if job.is_cancelled() { bail!("下载已取消"); }
		let size: u64 = tokio::fs::metadata(&temporary).await?.len();
		if size == 0 { bail!("下载的视频为空"); }
		tokio::fs::rename(&temporary, &output).await?;
		let size: i64 = size.try_into()?;
		lock(&self.db)?.execute("UPDATE downloads SET status='completed', downloaded_bytes=?2, total_bytes=?2, merge_progress=100, error='' WHERE id=?1 AND status!='deleting'", params![id, size])?;
		Ok(())
	}

	async fn download_mp4(&self, id: i64, mut response: Response, path: &Path, job: &Job) -> Result<()> {
		let total: Option<u64> = response.content_length();
		let mut file: File = File::create(path).await?;
		let mut downloaded: u64 = 0;
		let mut updated: Instant = Instant::now();
		let mut prefix: Vec<u8> = Vec::new();
		self.progress(id, 0, total, 0, 0, "downloading", None)?;
		loop {
			let chunk: Option<bytes::Bytes> = tokio::select! {
				_ = job.cancelled() => bail!("下载已取消"),
				chunk = response.chunk() => chunk?,
			};
			let Some(chunk) = chunk else { break; };
			if prefix.len() < 12 { prefix.extend(chunk.iter().take(12 - prefix.len())); }
			file.write_all(&chunk).await?;
			downloaded += chunk.len() as u64;
			if updated.elapsed() >= Duration::from_millis(500) {
				self.progress(id, downloaded, total, 0, 0, "downloading", None)?;
				updated = Instant::now();
			}
		}
		if total.is_some_and(|size| size != downloaded) { bail!("视频下载不完整，请重试"); }
		if prefix.get(4..8) != Some(b"ftyp".as_slice()) { bail!("服务器返回的内容不是 MP4 视频"); }
		file.flush().await?;
		file.sync_all().await?;
		Ok(())
	}
}

fn identity(page_url: &str) -> Result<(String, String, String)> {
	let url: Url = Url::parse(page_url)?;
	let path: Vec<&str> = url.path().trim_matches('/').split('/').collect();
	if url.origin().ascii_serialization() != "https://lie.ygopro3.cn" || path.len() != 4 || path[0] != "anime" || path[2] != "play" {
		bail!("无效的剧集页面地址");
	}
	let anime: u64 = path[1].parse()?;
	let episode: u64 = path[3].parse()?;
	let source: String = url.query_pairs().find(|(key, _)| key == "source").map(|(_, value)| value.into_owned()).unwrap_or_default();
	Ok((anime.to_string(), episode.to_string(), source))
}

#[tauri::command]
pub fn list_downloads(state: State<'_, Arc<Downloads>>) -> Result<Vec<Download>, String> {
	state.list().map_err(|error: anyhow::Error| format!("{error:#}"))
}

#[tauri::command]
pub async fn start_download(_app: AppHandle, request: DownloadRequest, state: State<'_, Arc<Downloads>>) -> Result<i64, String> {
	#[cfg(target_os = "android")]
	android_background::prepare(&_app).await.map_err(|error: anyhow::Error| format!("{error:#}"))?;
	let id: i64 = state.inner().start(request).map_err(|error: anyhow::Error| format!("{error:#}"))?;
	#[cfg(target_os = "android")]
	if let Err(error) = android_background::track(&_app, id).await {
		let _ = state.cancel(id, "无法更新安卓后台下载服务，请重试");
		return Err(format!("无法更新安卓后台下载服务：{error:#}"));
	}
	Ok(id)
}

#[tauri::command]
pub async fn take_download_notification(_app: AppHandle) -> Result<bool, String> {
	#[cfg(target_os = "android")]
	return android_background::take_open(&_app).await.map_err(|error: anyhow::Error| format!("{error:#}"));
	#[cfg(not(target_os = "android"))]
	Ok(false)
}

#[tauri::command]
pub async fn delete_download(id: i64, state: State<'_, Arc<Downloads>>) -> Result<(), String> {
	delete(state.inner().clone(), id).await.map_err(|error: anyhow::Error| format!("{error:#}"))
}

async fn delete(manager: Arc<Downloads>, id: i64) -> Result<()> {
	let job: Option<Arc<Job>> = {
		let jobs: MutexGuard<'_, HashMap<i64, Arc<Job>>> = lock(&manager.jobs)?;
		let changed: usize = lock(&manager.db)?.execute("UPDATE downloads SET status='deleting' WHERE id=?1", [id])?;
		if changed == 0 { return Ok(()); }
		let job: Option<Arc<Job>> = jobs.get(&id).cloned();
		if let Some(job) = &job { job.cancel.send_replace(true); }
		job
	};
	if let Some(job) = job {
		let mut done: watch::Receiver<bool> = job.finished.subscribe();
		while !*done.borrow_and_update() { done.changed().await?; }
	}
	manager.remove_files(id)?;
	lock(&manager.db)?.execute("DELETE FROM downloads WHERE id=?1", [id])?;
	Ok(())
}

#[tauri::command]
pub fn cached_video(page_url: String, cover_url: Option<String>, state: State<'_, Arc<Downloads>>) -> Result<Option<String>, String> {
	let result: Result<Option<String>> = (|| {
		if let Some(item) = state.find(&page_url)? {
			if let Some(cover) = cover_url.as_deref().filter(|cover| !cover.is_empty()) {
				lock(&state.db)?.execute("UPDATE downloads SET cover_url=?2 WHERE anime_id=?1", params![item.anime_id, cover])?;
			}
			if item.status == "completed" {
				let path: PathBuf = state.file(item.id);
				if path.is_file() { return Ok(Some(path.to_string_lossy().into_owned())); }
				state.fail(item.id, "缓存文件已丢失，请重试")?;
			}
		}
		Ok(None)
	})();
	result.map_err(|error: anyhow::Error| format!("{error:#}"))
}

#[tauri::command]
pub async fn cached_video_bytes(page_url: String, offset: u64, state: State<'_, Arc<Downloads>>) -> Result<IpcResponse, String> {
	if !cfg!(target_os = "android") {
		return Err("仅安卓端通过二进制数据读取缓存视频".into());
	}
	let manager: Arc<Downloads> = Arc::clone(state.inner());
	let bytes: Vec<u8> = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<u8>> {
		let path: PathBuf = manager.completed_file(&page_url)?;
		let mut file: fs::File = fs::File::open(&path).context("无法读取本地缓存视频")?;
		let total: u64 = file.metadata()?.len();
		if offset >= total { bail!("缓存视频读取位置超出文件范围"); }
		// Android's JNI response is copied into a Java byte array. Never send the whole video.
		let length: usize = (total - offset).min(200 * 1024 * 1024).try_into()?;
		let mut bytes: Vec<u8> = vec![0; length];
		file.seek(SeekFrom::Start(offset))?;
		file.read_exact(&mut bytes).context("缓存视频读取不完整，请检查文件是否被删除或修改")?;
		Ok(bytes)
	}).await.map_err(|error: tauri::Error| error.to_string())?
		.map_err(|error: anyhow::Error| format!("{error:#}"))?;
	Ok(IpcResponse::new(bytes))
}

#[tauri::command]
pub fn cached_video_size(page_url: String, state: State<'_, Arc<Downloads>>) -> Result<u64, String> {
	let result: Result<u64> = (|| {
		let path: PathBuf = state.completed_file(&page_url)?;
		let size: u64 = fs::metadata(path).context("无法读取缓存视频大小")?.len();
		if size == 0 { bail!("本地缓存视频为空，请重新缓存"); }
		Ok(size)
	})();
	result.map_err(|error: anyhow::Error| format!("{error:#}"))
}

#[tauri::command]
pub fn download_directory(state: State<'_, Arc<Downloads>>) -> String {
	state.directory.to_string_lossy().into_owned()
}

#[tauri::command]
pub fn get_watch_position(page_url: String, state: State<'_, Arc<Downloads>>) -> Result<f64, String> {
	let result: Result<f64> = (|| {
		let (anime, episode, _): (String, String, String) = identity(&page_url)?;
		let position: Option<f64> = lock(&state.db)?.query_row(
			"SELECT position_seconds FROM watch_positions WHERE anime_id=?1 AND episode_id=?2",
			params![anime, episode], |row: &Row<'_>| row.get(0),
		).optional()?;
		Ok(position.unwrap_or(0.0))
	})();
	result.map_err(|error: anyhow::Error| format!("{error:#}"))
}

#[tauri::command]
pub fn save_watch_position(page_url: String, position: f64, duration: f64, state: State<'_, Arc<Downloads>>) -> Result<(), String> {
	let result: Result<()> = (|| {
		if !position.is_finite() || !duration.is_finite() || position < 0.0 || duration <= 0.0 {
			bail!("无效的观看位置或视频时长");
		}
		let (anime, episode, _): (String, String, String) = identity(&page_url)?;
		let position: f64 = if position >= duration { 0.0 } else { position };
		lock(&state.db)?.execute(
			"INSERT INTO watch_positions (anime_id, episode_id, position_seconds, duration_seconds)
			VALUES (?1,?2,?3,?4) ON CONFLICT(anime_id, episode_id) DO UPDATE SET
			position_seconds=excluded.position_seconds, duration_seconds=excluded.duration_seconds, updated_at=unixepoch()",
			params![anime, episode, position, duration],
		)?;
		Ok(())
	})();
	result.map_err(|error: anyhow::Error| format!("{error:#}"))
}
