//! HTTP helpers for the custom protocols: small fetches and range requests.

use std::sync::OnceLock;
use std::time::Duration;

use crate::error::AppError;

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, AppError> {
    let response = http_client()
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::network(format!("fetch media: {e}")))?;
    if !response.status().is_success() {
        return Err(AppError::network(format!(
            "fetch media: HTTP {}",
            response.status()
        )));
    }
    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| AppError::network(format!("read media: {e}")))
}

/// Returns `(status, body, content_range, content_type)` as sent by the server.
pub async fn fetch_range(
    url: &str,
    range: Option<&str>,
) -> Result<(u16, Vec<u8>, Option<String>, Option<String>), AppError> {
    let mut request = http_client().get(url);
    if let Some(range) = range {
        request = request.header(reqwest::header::RANGE, range);
    }
    let response = request
        .send()
        .await
        .map_err(|e| AppError::network(format!("fetch media: {e}")))?;
    let status = response.status().as_u16();
    let content_range = response
        .headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let body = response
        .bytes()
        .await
        .map_err(|e| AppError::network(format!("read media: {e}")))?
        .to_vec();
    Ok((status, body, content_range, content_type))
}
