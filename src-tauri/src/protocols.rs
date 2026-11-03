//! `thumb://`, `media://` and `local://` response handlers.

use std::sync::Arc;

use source_core::error::{AppError, ErrorCode};
use source_core::{LOCAL_THUMB_EDGE, fetch_bytes, fetch_range, preview_from_jpeg};
use tauri::http::{Response, StatusCode, header};
use tauri::{AppHandle, Manager, UriSchemeResponder};

use crate::state::AppState;

pub(crate) fn http_response(
    status: u16,
    content_type: &str,
    body: Vec<u8>,
    content_range: Option<String>,
    cache_control: &str,
) -> Response<Vec<u8>> {
    let mut builder = Response::builder()
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK))
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header("Cross-Origin-Resource-Policy", "cross-origin")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, cache_control);
    if let Some(range) = content_range {
        builder = builder.header(header::CONTENT_RANGE, range);
    }
    builder
        .body(body)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

pub(crate) fn error_response(error: &AppError) -> Response<Vec<u8>> {
    let status = match error.code {
        ErrorCode::NotFound => 404,
        ErrorCode::Unauthorized => 401,
        ErrorCode::Permission => 403,
        _ => 502,
    };
    http_response(
        status,
        "application/json",
        serde_json::json!({ "error": error.message })
            .to_string()
            .into_bytes(),
        None,
        "no-store",
    )
}

pub(crate) fn thumb(
    app: AppHandle,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let id = urlencoding::decode(request.uri().path().trim_start_matches('/'))
        .map(|v| v.to_string())
        .unwrap_or_default();
    tauri::async_runtime::spawn(async move {
        let (core, cache) = {
            let state = app.state::<AppState>();
            (Arc::clone(&state.core), Arc::clone(&state.cache))
        };
        if let Some(bytes) = cache.get(&id) {
            responder.respond(http_response(
                200,
                "image/jpeg",
                bytes,
                None,
                "public, max-age=86400",
            ));
            return;
        }
        match core.browse.thumb_url(&id).await {
            Ok(url) => match fetch_bytes(&url).await {
                Ok(bytes) => {
                    let _ = cache.put(&id, &bytes);
                    // Derive the blur-up placeholder from the JPEG we already
                    // downloaded, so it never costs an extra request. Done
                    // before responding so the mosaic lands on first paint.
                    let mosaic = match core.browse.asset_thumb_hash(&id).await {
                        Ok(Some(_)) => None,
                        _ => preview_from_jpeg(&bytes),
                    };
                    responder.respond(http_response(
                        200,
                        "image/jpeg",
                        bytes,
                        None,
                        "public, max-age=86400",
                    ));
                    if let Some(mosaic) = mosaic {
                        let _ = core.browse.set_asset_thumb_hash(&id, &mosaic).await;
                    }
                }
                Err(error) => responder.respond(error_response(&error)),
            },
            Err(error) => responder.respond(error_response(&error)),
        }
    });
}

pub(crate) fn media(
    app: AppHandle,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let id = urlencoding::decode(request.uri().path().trim_start_matches('/'))
        .map(|v| v.to_string())
        .unwrap_or_default();
    let range = request
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    tauri::async_runtime::spawn(async move {
        let core = {
            let state = app.state::<AppState>();
            Arc::clone(&state.core)
        };
        match core.browse.media_url(&id).await {
            Ok(url) => match fetch_range(&url, range.as_deref()).await {
                Ok((status, body, content_range, content_type)) => {
                    responder.respond(http_response(
                        status,
                        content_type
                            .as_deref()
                            .unwrap_or("application/octet-stream"),
                        body,
                        content_range,
                        "no-store",
                    ))
                }
                Err(error) => responder.respond(error_response(&error)),
            },
            Err(error) => responder.respond(error_response(&error)),
        }
    });
}

/// Cache key that follows the file on disk: a rewrite bumps the mtime and
/// therefore the key, so a stale JPEG is never served.
fn local_thumb_key(path: &str) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|age| age.as_millis())
        .unwrap_or(0);
    Some(format!("local:{LOCAL_THUMB_EDGE}:{modified}:{path}"))
}

fn local_thumb_response(bytes: Vec<u8>) -> Response<Vec<u8>> {
    http_response(200, "image/jpeg", bytes, None, "public, max-age=86400")
}

/// `local://` — thumbnails for files outside the Cloudreve catalog.
///
/// Streaming the original bytes here read multi-megabyte photos (and whole
/// videos) on the UI thread, which froze the window while a folder painted.
/// This answers with a cached 512px JPEG built by the local media port, so the
/// file itself is only ever read by the platform adapter.
pub(crate) fn local_thumb(
    app: AppHandle,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let raw = urlencoding::decode(request.uri().path().trim_start_matches('/'))
        .map(|v| v.to_string())
        .unwrap_or_default();
    let path = raw.strip_prefix("file://").unwrap_or(&raw).to_string();
    tauri::async_runtime::spawn(async move {
        let (core, cache) = {
            let state = app.state::<AppState>();
            (Arc::clone(&state.core), Arc::clone(&state.cache))
        };
        let Some(key) = local_thumb_key(&path) else {
            responder.respond(http_response(
                404,
                "text/plain",
                Vec::new(),
                None,
                "no-store",
            ));
            return;
        };
        if let Some(bytes) = cache.get(&key) {
            responder.respond(local_thumb_response(bytes));
            return;
        }
        let source = path.clone();
        let thumb = tauri::async_runtime::spawn_blocking(move || {
            core.browse.local_thumb(&source, LOCAL_THUMB_EDGE).ok()
        })
        .await
        .ok()
        .flatten();
        match thumb {
            Some(bytes) => {
                let _ = cache.put(&key, &bytes);
                responder.respond(local_thumb_response(bytes));
            }
            None => responder.respond(http_response(
                415,
                "text/plain",
                Vec::new(),
                None,
                "no-store",
            )),
        }
    });
}
