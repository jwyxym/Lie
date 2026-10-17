use aes::{Aes128, cipher::{Block, BlockDecrypt, KeyInit}};
use hls_transmux::{ByteRange, Error, Result, Source, SourceLocation, TextResource};
use regex::Regex;
use reqwest::{Client, Response, StatusCode};
use std::{collections::HashMap, future::Future, pin::Pin, sync::{LazyLock, Mutex}};
use tauri::Url;

static ATTRIBUTES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([A-Z0-9-]+)=("[^"]*"|[^,]*)"#).expect("valid HLS attribute pattern"));

#[derive(Clone, Debug)]
struct Encryption {
	key: [u8; 16],
	iv: Option<[u8; 16]>,
}

#[derive(Clone, Debug)]
struct Segment {
	location: SourceLocation,
	encryption: Encryption,
	iv: [u8; 16],
}

/// Decrypt standard AES-128 HLS before passing segments to the MP4 muxer.
/// Each segment gets a distinct internal URL, including when byte ranges reuse a resource.
#[derive(Debug)]
pub struct HlsSource {
	client: Client,
	segments: Mutex<HashMap<String, Segment>>,
}

impl HlsSource {
	pub fn new(client: Client) -> Self {
		Self { client, segments: Mutex::new(HashMap::new()) }
	}

	async fn fetch(&self, location: &SourceLocation, range: Option<&ByteRange>) -> Result<(Vec<u8>, Url)> {
		let url: &Url = match location {
			SourceLocation::Url(url) if matches!(url.scheme(), "http" | "https") => url,
			_ => return Err(Error::invalid("仅支持 HTTP/HTTPS 视频源")),
		};
		let header: Option<String> = if let Some(range) = range {
			let end: u64 = range.offset.checked_add(range.length).and_then(|end| end.checked_sub(1)).ok_or_else(|| Error::invalid("无效 HLS 字节范围"))?;
			Some(format!("bytes={}-{end}", range.offset))
		} else { None };
		let mut response: Response = crate::media::get(&self.client, url, header.as_deref()).await.map_err(|error| Error::Http(format!("{error:#}")))?;
		if let Some(range) = range {
			let prefix: String = format!("bytes {}-{}/", range.offset, range.offset + range.length - 1);
			if response.status() != StatusCode::PARTIAL_CONTENT
				|| !response.headers().get("Content-Range").and_then(|value| value.to_str().ok()).is_some_and(|value| value.starts_with(&prefix)) {
				return Err(Error::invalid("视频服务器未返回所需字节范围"));
			}
		}
		let final_url: Url = response.url().clone();
		let mut bytes: Vec<u8> = Vec::new();
		while let Some(chunk) = response.chunk().await.map_err(|error| Error::Http(error.to_string()))? {
			if bytes.len() + chunk.len() > 128 * 1024 * 1024 { return Err(Error::invalid("单个 HLS 分片超过 128 MB")); }
			bytes.extend_from_slice(&chunk);
		}
		if range.is_some_and(|range| range.length != bytes.len() as u64) { return Err(Error::invalid("HLS 分片下载不完整")); }
		Ok((bytes, final_url))
	}

	async fn playlist(&self, mut resource: TextResource) -> Result<TextResource> {
		let base: Url = match &resource.location {
			SourceLocation::Url(url) => url.clone(),
			SourceLocation::File(_) => return Err(Error::invalid("仅支持 HTTP/HTTPS 视频源")),
		};
		let mut encryption: Option<Encryption> = None;
		let mut keys: HashMap<String, [u8; 16]> = HashMap::new();
		let mut sequence: u128 = 0;
		let mut output: String = String::new();
		let mut segment_index: usize = 0;
		for raw in resource.content.lines() {
			let line: &str = raw.trim();
			if let Some(value) = line.strip_prefix("#EXT-X-MEDIA-SEQUENCE:") {
				sequence = value.parse().map_err(|_| Error::invalid("无效 HLS 分片序号"))?;
			}
			if let Some(value) = line.strip_prefix("#EXT-X-KEY:") {
				let attributes: HashMap<String, String> = ATTRIBUTES.captures_iter(value)
					.map(|item: regex::Captures<'_>| (item[1].to_string(), item[2].trim_matches('"').to_string())).collect();
				match attributes.get("METHOD").map(String::as_str) {
					Some("NONE") => encryption = None,
					Some("AES-128") => {
						if attributes.get("KEYFORMAT").is_some_and(|format| format != "identity") {
							return Err(Error::unsupported("不支持 DRM 加密视频缓存"));
						}
						let uri: &String = attributes.get("URI").ok_or_else(|| Error::invalid("HLS 缺少密钥地址"))?;
						let key_url: Url = base.join(uri).map_err(|error| Error::invalid(error.to_string()))?;
						let key: [u8; 16] = if let Some(key) = keys.get(key_url.as_str()) { *key } else {
							let (bytes, _): (Vec<u8>, Url) = self.fetch(&SourceLocation::Url(key_url.clone()), None).await?;
							let key: [u8; 16] = bytes.try_into().map_err(|_| Error::invalid("AES-128 密钥长度不是 16 字节"))?;
							keys.insert(key_url.to_string(), key);
							key
						};
						let iv: Option<[u8; 16]> = attributes.get("IV").map(|value: &String| -> Result<[u8; 16]> {
							let hex: &str = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")).ok_or_else(|| Error::invalid("无效 HLS IV"))?;
							let number: u128 = u128::from_str_radix(hex, 16).map_err(|_| Error::invalid("无效 HLS IV"))?;
							Ok(number.to_be_bytes())
						}).transpose()?;
						encryption = Some(Encryption { key, iv });
					}
					_ => return Err(Error::unsupported("不支持该 HLS 加密方式（支持 AES-128）")),
				}
				continue;
			}
			if line.starts_with("#EXT-X-MAP:") && encryption.is_some() {
				return Err(Error::unsupported("暂不支持加密的 fMP4 初始化片段"));
			}
			if !line.is_empty() && !line.starts_with('#') {
				if let Some(encryption) = &encryption {
					let location: Url = base.join(line).map_err(|error| Error::invalid(error.to_string()))?;
					let mut internal: Url = location.clone();
					internal.set_fragment(Some(&format!("lie-cache-{segment_index}")));
					let segment: Segment = Segment {
						location: SourceLocation::Url(location),
						encryption: encryption.clone(),
						iv: encryption.iv.unwrap_or(sequence.to_be_bytes()),
					};
					self.segments.lock().map_err(|_| Error::invalid("HLS 缓存状态不可用"))?.insert(internal.to_string(), segment);
					output.push_str(internal.as_str());
					output.push('\n');
				} else { output.push_str(line); output.push('\n'); }
				sequence += 1;
				segment_index += 1;
			} else { output.push_str(line); output.push('\n'); }
		}
		resource.content = output;
		Ok(resource)
	}
}

impl Source for HlsSource {
	fn read_text<'a>(&'a self, location: &'a SourceLocation) -> Pin<Box<dyn Future<Output = Result<TextResource>> + Send + 'a>> {
		Box::pin(async move {
			let (bytes, url): (Vec<u8>, Url) = self.fetch(location, None).await?;
			let content: String = String::from_utf8(bytes).map_err(|_| Error::invalid("m3u8 不是有效 UTF-8 文本"))?;
			self.playlist(TextResource { content, location: SourceLocation::Url(url) }).await
		})
	}

	fn read_bytes<'a>(&'a self, location: &'a SourceLocation, range: Option<&'a ByteRange>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
		Box::pin(async move {
			let segment: Option<Segment> = match location {
				SourceLocation::Url(url) => self.segments.lock().map_err(|_| Error::invalid("HLS 缓存状态不可用"))?.get(url.as_str()).cloned(),
				SourceLocation::File(_) => None,
			};
			if let Some(segment) = segment {
				if range.is_some() { return Err(Error::unsupported("暂不支持 AES-128 加密的字节范围分片")); }
				let (mut bytes, _): (Vec<u8>, Url) = self.fetch(&segment.location, range).await?;
				decrypt(&mut bytes, &segment.encryption.key, segment.iv)?;
				Ok(bytes)
			} else {
				let (bytes, _): (Vec<u8>, Url) = self.fetch(location, range).await?;
				Ok(bytes)
			}
		})
	}
}

fn decrypt(bytes: &mut Vec<u8>, key: &[u8; 16], mut iv: [u8; 16]) -> Result<()> {
	if bytes.is_empty() || bytes.len() % 16 != 0 { return Err(Error::invalid("AES-128 分片长度无效")); }
	let cipher: Aes128 = Aes128::new_from_slice(key).map_err(|_| Error::invalid("无效密钥"))?;
	for chunk in bytes.chunks_exact_mut(16) {
		let next: [u8; 16] = <[u8; 16]>::try_from(&*chunk).map_err(|_| Error::invalid("无效密文块"))?;
		let mut block: Block<Aes128> = Block::<Aes128>::default();
		block.copy_from_slice(chunk);
		cipher.decrypt_block(&mut block);
		chunk.copy_from_slice(&block);
		for (value, previous) in chunk.iter_mut().zip(iv) { *value ^= previous; }
		iv = next;
	}
	let padding: usize = usize::from(*bytes.last().ok_or_else(|| Error::invalid("空 AES 分片"))?);
	if padding == 0 || padding > 16 || !bytes[bytes.len() - padding..].iter().all(|value| usize::from(*value) == padding) {
		return Err(Error::invalid("AES-128 分片校验失败"));
	}
	bytes.truncate(bytes.len() - padding);
	Ok(())
}
