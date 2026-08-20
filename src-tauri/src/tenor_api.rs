//! Tenor GIF API client (server-side).
//! The API key lives here, NOT in the frontend bundle.
//! This prevents key extraction and enables SSRF protection + rate limiting.

use serde::{Deserialize, Serialize};
use std::time::Duration;

const TENOR_API_BASE: &str = "https://g.tenor.com/v1";
const TENOR_API_KEY_ENV: &str = "TENOR_API_KEY";
const DEFAULT_API_KEY: &str = "LIVDSRZULELA"; // fallback if env not set

/// GIF data returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifResult {
    pub id: String,
    pub title: String,
    pub preview_url: String,
    pub full_url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
struct TenorV1Response {
    results: Vec<TenorV1Result>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct TenorV1Result {
    id: String,
    title: Option<String>,
    content_description: Option<String>,
    media: Vec<TenorV1Media>,
}

#[derive(Deserialize)]
struct TenorV1Media {
    nanogif: Option<TenorV1Format>,
    tinygif: Option<TenorV1Format>,
    mediumgif: Option<TenorV1Format>,
    gif: Option<TenorV1Format>,
}

#[derive(Deserialize)]
struct TenorV1Format {
    url: String,
    dims: [u32; 2],
    size: u32,
}

fn api_key() -> String {
    std::env::var(TENOR_API_KEY_ENV).unwrap_or_else(|_| DEFAULT_API_KEY.to_string())
}

/// Search GIFs via Tenor API (server-side proxy)
#[tauri::command]
pub async fn search_tenor(query: Option<String>, limit: Option<u32>) -> Result<Vec<GifResult>, String> {
    let key = api_key();
    let limit = limit.unwrap_or(30).min(50); // cap at 50
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;

    let url = if let Some(q) = query.as_ref().filter(|q| !q.trim().is_empty()) {
        format!("{TENOR_API_BASE}/search?key={key}&q={q}&limit={limit}&media_filter=minimal")
    } else {
        format!("{TENOR_API_BASE}/trending?key={key}&limit={limit}&media_filter=minimal")
    };

    let resp = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Network: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Tenor API HTTP {}", resp.status()));
    }

    let data: TenorV1Response = resp.json().await
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(data.results.into_iter().filter_map(|r| {
        let preview = r.media.first()?.nanogif.as_ref()
            .or_else(|| r.media.first()?.tinygif.as_ref())?;
        let full = r.media.first()?.tinygif.as_ref()
            .or_else(|| r.media.first()?.mediumgif.as_ref())
            .or_else(|| r.media.first()?.gif.as_ref())?;

        Some(GifResult {
            id: r.id,
            title: r.content_description.unwrap_or_else(|| r.title.unwrap_or_default()),
            preview_url: preview.url.clone(),
            full_url: full.url.clone(),
            width: preview.dims[0],
            height: preview.dims[1],
        })
    }).collect())
}