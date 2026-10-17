use anyhow::{Context, Result, bail};
use reqwest::{Client, Error as RequestError, Response, redirect::Policy};
use std::time::Duration;
use tauri::Url;

pub fn client() -> Result<Client> {
	Client::builder()
		.connect_timeout(Duration::from_secs(15))
		.read_timeout(Duration::from_secs(30))
		// Resolve redirects ourselves so errors identify the actual media server.
		.redirect(Policy::none())
		.referer(false)
		.build().context("无法创建视频下载客户端")
}

pub async fn get(client: &Client, url: &Url, range: Option<&str>) -> Result<Response> {
	let mut target: Url = url.clone();
	for _ in 0..=10 {
		if !matches!(target.scheme(), "http" | "https") {
			bail!("视频跳转使用了不支持的协议");
		}
		let address: String = format!("{}:{}", target.host_str().unwrap_or_default(), target.port_or_known_default().unwrap_or(0));
		let mut attempt: u32 = 0;
		let response: Response = loop {
			let mut request: reqwest::RequestBuilder = client.get(target.clone());
			if let Some(range) = range { request = request.header("Range", range); }
			let result: Result<Response, RequestError> = request.send().await;
			match result {
				Ok(response) => break response,
				Err(error) if attempt < 2 && (error.is_connect() || error.is_timeout()) => {
					attempt += 1;
					tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
				}
				Err(error) => return Err(error).with_context(|| format!("无法连接视频服务器 {address}；请检查该地址的网络是否可用")),
			}
		};
		if matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308) {
			let location: &str = response.headers().get("Location")
				.context("视频服务器返回跳转但没有 Location")?.to_str()?;
			target = target.join(location).context("视频跳转地址无效")?;
			continue;
		}
		return response.error_for_status().with_context(|| format!("视频服务器 {address} 返回错误状态"));
	}
	bail!("视频地址跳转次数超过 10 次")
}
