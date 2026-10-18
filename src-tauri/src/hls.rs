use aes::{Aes128, cipher::{Block, BlockDecrypt, KeyInit}};
use anyhow::{Context, Result, bail, ensure};
use m3u8_rs::{ByteRange, Key, KeyMethod, Map, MediaPlaylist, Playlist, VariantStream};
use mp4forge::mux::{MuxRequest, MuxTrackSpec, inspect::inspect_direct_ingest_path_async, mux_to_path_async_with_progress};
use regex::Regex;
use reqwest::{Client, Response, StatusCode};
use std::{collections::HashMap, future::Future, path::{Path, PathBuf}, pin::Pin, sync::{Arc, LazyLock, Mutex}, time::{Duration, Instant}};
use tauri::Url;
use tokio::{fs::File, io::AsyncWriteExt};

static NONE_KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^#EXT-X-KEY:METHOD=NONE\s*$").expect("valid key pattern"));

pub trait CancelToken: Send + Sync {
	fn is_cancelled(&self) -> bool;
	fn cancelled(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

pub struct Progress {
	pub downloaded_bytes: u64,
	pub completed_segments: usize,
	pub total_segments: usize,
	pub merge_progress: Option<u8>,
}

struct Downloader {
	client: Client,
}

impl Downloader {
	async fn body(&self, mut response: Response, range: Option<(u64, u64)>, limit: usize) -> Result<(Vec<u8>, Url)> {
		ensure!(response.status().is_success(), "视频服务器返回 {}", response.status());
		if let Some((offset, length)) = range {
			let end: u64 = offset.checked_add(length).and_then(|value: u64| value.checked_sub(1)).context("无效字节范围")?;
			let prefix: String = format!("bytes {offset}-{end}/");
			ensure!(response.status() == StatusCode::PARTIAL_CONTENT
				&& response.headers().get("content-range").and_then(|value| value.to_str().ok()).is_some_and(|value: &str| value.starts_with(&prefix)),
				"视频服务器未返回所需字节范围 {prefix}");
		} else {
			ensure!(response.status() != StatusCode::PARTIAL_CONTENT, "视频服务器返回了非预期的部分内容");
		}
		let url: Url = response.url().clone();
		let expected: Option<u64> = response.content_length();
		ensure!(expected.is_none_or(|size: u64| size <= limit as u64), "HLS 单个资源超过大小限制");
		let mut bytes: Vec<u8> = Vec::new();
		while let Some(chunk) = response.chunk().await.context("读取 HLS 响应失败")? {
			ensure!(chunk.len() <= limit.saturating_sub(bytes.len()), "HLS 单个资源超过大小限制");
			bytes.extend_from_slice(&chunk);
		}
		ensure!(expected.is_none_or(|size: u64| size == bytes.len() as u64), "HLS 资源下载不完整");
		ensure!(range.is_none_or(|(_, length): (u64, u64)| length == bytes.len() as u64), "HLS 字节范围下载不完整");
		Ok((bytes, url))
	}

	async fn fetch(&self, url: &Url, range: Option<(u64, u64)>, limit: usize) -> Result<(Vec<u8>, Url)> {
		ensure!(matches!(url.scheme(), "http" | "https"), "仅支持 HTTP/HTTPS 视频资源");
		let header: Option<String> = range.map(|(offset, length): (u64, u64)| -> Result<String> {
			let end: u64 = offset.checked_add(length).and_then(|value: u64| value.checked_sub(1)).context("无效 HLS 字节范围")?;
			Ok(format!("bytes={offset}-{end}"))
		}).transpose()?;
		let response: Response = crate::media::get(&self.client, url, header.as_deref()).await?;
		self.body(response, range, limit).await
	}

	async fn playlist(&self, response: Response) -> Result<(MediaPlaylist, Url)> {
		let (mut bytes, mut base): (Vec<u8>, Url) = self.body(response, None, 4 * 1024 * 1024).await?;
		for _ in 0..8 {
			let text: &str = std::str::from_utf8(&bytes).context("m3u8 不是有效 UTF-8 文本")?.trim_start_matches('\u{feff}');
			ensure!(text.trim_start().starts_with("#EXTM3U"), "服务器未返回 m3u8 播放列表");
			// m3u8-rs 6 rejects METHOD=NONE without IV. IV is ignored for NONE.
			let normalized: String = NONE_KEY.replace_all(text, "#EXT-X-KEY:METHOD=NONE,IV=0x0").into_owned();
			let key_count: usize = normalized.lines().filter(|line: &&str| line.trim_start().starts_with("#EXT-X-KEY:")).count();
			let (remaining, playlist): (&[u8], Playlist) = m3u8_rs::parse_playlist(normalized.as_bytes())
				.map_err(|_| anyhow::anyhow!("m3u8 播放列表格式无效"))?;
			ensure!(remaining.iter().all(u8::is_ascii_whitespace), "m3u8 存在未能解析的内容");
			match playlist {
				Playlist::MediaPlaylist(playlist) => {
					ensure!(playlist.segments.iter().filter(|segment| segment.key.is_some()).count() == key_count,
						"m3u8 含无法解析的加密声明");
					ensure!(playlist.end_list && !playlist.segments.is_empty(), "仅支持已结束且非空的 m3u8 视频");
					ensure!(!playlist.i_frames_only, "不支持仅含关键帧的播放列表");
					ensure!(!playlist.segments.iter().any(|segment| segment.discontinuity), "暂不支持时间轴不连续的 m3u8 分片");
					return Ok((playlist, base));
				}
				Playlist::MasterPlaylist(master) => {
					let variant: &VariantStream = master.variants.iter().filter(|item: &&VariantStream| !item.is_i_frame)
						.max_by_key(|item: &&VariantStream| item.bandwidth).context("m3u8 没有可用清晰度")?;
					ensure!(variant.audio.is_none(), "暂不支持独立音轨的 m3u8 缓存");
					let url: Url = base.join(&variant.uri)?;
					(bytes, base) = self.fetch(&url, None, 4 * 1024 * 1024).await?;
				}
			}
		}
		bail!("m3u8 嵌套层数过多")
	}
}

fn byte_range(url: &Url, range: Option<&ByteRange>, previous: &mut Option<(Url, u64)>) -> Result<Option<(u64, u64)>> {
	let Some(range) = range else { *previous = None; return Ok(None); };
	let offset: u64 = range.offset.or_else(|| previous.as_ref().filter(|(last, _): &&(Url, u64)| last == url).map(|(_, end): &(Url, u64)| *end))
		.context("HLS 字节范围省略偏移时必须紧接同一资源")?;
	ensure!(range.length > 0, "HLS 字节范围长度无效");
	let end: u64 = offset.checked_add(range.length).context("HLS 字节范围溢出")?;
	*previous = Some((url.clone(), end));
	Ok(Some((offset, range.length)))
}

pub async fn download(client: Client, response: Response, output: &Path, progress: Arc<dyn Fn(Progress) + Send + Sync>) -> Result<()> {
	let downloader: Downloader = Downloader { client };
	let (playlist, base): (MediaPlaylist, Url) = downloader.playlist(response).await?;
	let staging: PathBuf = output.with_file_name("segments.media");
	let mut file: File = File::create(&staging).await?;
	let total: usize = playlist.segments.len();
	let mut downloaded: u64 = 0;
	let mut previous_range: Option<(Url, u64)> = None;
	let mut active_key: Option<Key> = None;
	let mut active_map: Option<Map> = None;
	let mut keys: HashMap<String, [u8; 16]> = HashMap::new();
	progress(Progress { downloaded_bytes: 0, completed_segments: 0, total_segments: total, merge_progress: None });
	for (index, segment) in playlist.segments.iter().enumerate() {
		if let Some(key) = &segment.key { active_key = Some(key.clone()); }
		if let Some(map) = &segment.map {
			if let Some(previous) = &active_map {
				ensure!(previous == map, "暂不支持中途更换初始化片段的 m3u8");
			} else {
				ensure!(index == 0, "m3u8 初始化片段必须位于首个分片之前");
				ensure!(active_key.as_ref().is_none_or(|key: &Key| key.method == KeyMethod::None), "暂不支持加密 fMP4 初始化片段");
				let url: Url = base.join(&map.uri)?;
				let range: Option<(u64, u64)> = byte_range(&url, map.byte_range.as_ref(), &mut None)?;
				let (bytes, _): (Vec<u8>, Url) = downloader.fetch(&url, range, 128 * 1024 * 1024).await?;
				file.write_all(&bytes).await?;
				downloaded += bytes.len() as u64;
				active_map = Some(map.clone());
			}
		}
		let url: Url = base.join(&segment.uri)?;
		let range: Option<(u64, u64)> = byte_range(&url, segment.byte_range.as_ref(), &mut previous_range)?;
		let encryption: Option<([u8; 16], [u8; 16])> = if let Some(key) = active_key.as_ref().filter(|key: &&Key| key.method != KeyMethod::None) {
			ensure!(key.method == KeyMethod::AES128, "仅支持 AES-128 加密分片");
			ensure!(key.keyformat.as_deref().is_none_or(|format: &str| format == "identity"), "不支持 DRM 加密视频缓存");
			ensure!(range.is_none() && active_map.is_none(), "暂不支持加密的字节范围或 fMP4 分片");
			let key_url: Url = base.join(key.uri.as_deref().context("HLS 缺少密钥地址")?)?;
			let secret: [u8; 16] = if let Some(secret) = keys.get(key_url.as_str()) { *secret } else {
				let (bytes, _): (Vec<u8>, Url) = downloader.fetch(&key_url, None, 16).await?;
				let secret: [u8; 16] = bytes.try_into().map_err(|_| anyhow::anyhow!("AES-128 密钥长度不是 16 字节"))?;
				keys.insert(key_url.to_string(), secret);
				secret
			};
			let sequence: u64 = playlist.media_sequence.checked_add(u64::try_from(index)?).context("HLS 分片序号溢出")?;
			let iv: [u8; 16] = if let Some(value) = &key.iv {
				let hex: &str = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")).context("无效 HLS IV")?;
				u128::from_str_radix(hex, 16).context("无效 HLS IV")?.to_be_bytes()
			} else { u128::from(sequence).to_be_bytes() };
			Some((secret, iv))
		} else { None };
		let (mut bytes, _): (Vec<u8>, Url) = downloader.fetch(&url, range, 128 * 1024 * 1024).await?;
		downloaded += bytes.len() as u64;
		if let Some((secret, iv)) = encryption { decrypt(&mut bytes, &secret, iv)?; }
		ensure!(!bytes.is_empty(), "HLS 分片内容为空");
		file.write_all(&bytes).await?;
		if index == 0 {
			file.flush().await?;
			inspect_direct_ingest_path_async(&staging).await.context("首个 m3u8 分片无法封装 MP4")?;
		}
		progress(Progress { downloaded_bytes: downloaded, completed_segments: index + 1, total_segments: total, merge_progress: None });
	}
	file.flush().await?;
	drop(file);
	let request: MuxRequest = MuxRequest::new(vec![MuxTrackSpec::path(&staging)]);
	let merge_state: Mutex<Option<(u8, Instant)>> = Mutex::new(None);
	let on_merge = |written: u64, length: u64| {
		if length == 0 { return; }
		// Keep 100% for a completed, flushed and published cache file.
		let percent: u8 = (u128::from(written) * 100 / u128::from(length)).min(99) as u8;
		let Ok(mut state) = merge_state.lock() else { return; };
		if state.as_ref().is_some_and(|(last, time): &(u8, Instant)|
			*last == percent || (time.elapsed() < Duration::from_millis(250) && percent < 99)) { return; }
		*state = Some((percent, Instant::now()));
		drop(state);
		progress(Progress { downloaded_bytes: downloaded, completed_segments: total, total_segments: total, merge_progress: Some(percent) });
	};
	mux_to_path_async_with_progress(&request, output, &on_merge).await.context("m3u8 封装 MP4 失败")?;
	tokio::fs::remove_file(staging).await?;
	Ok(())
}

fn decrypt(bytes: &mut Vec<u8>, key: &[u8; 16], mut iv: [u8; 16]) -> Result<()> {
	ensure!(!bytes.is_empty() && bytes.len() % 16 == 0, "AES-128 分片长度无效");
	let cipher: Aes128 = Aes128::new_from_slice(key).map_err(|_| anyhow::anyhow!("无效密钥"))?;
	for chunk in bytes.chunks_exact_mut(16) {
		let next: [u8; 16] = <[u8; 16]>::try_from(&*chunk)?;
		let mut block: Block<Aes128> = Block::<Aes128>::default();
		block.copy_from_slice(chunk);
		cipher.decrypt_block(&mut block);
		chunk.copy_from_slice(&block);
		for (value, previous) in chunk.iter_mut().zip(iv) { *value ^= previous; }
		iv = next;
	}
	let padding: usize = usize::from(*bytes.last().context("空 AES 分片")?);
	ensure!(padding > 0 && padding <= 16 && bytes[bytes.len() - padding..].iter().all(|value: &u8| usize::from(*value) == padding), "AES-128 分片校验失败");
	bytes.truncate(bytes.len() - padding);
	Ok(())
}
