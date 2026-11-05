//! The `media://` response handler and the addresses the webview loads by.

use std::sync::Arc;

use source_core::error::{AppError, ErrorCode};
use source_core::{LOCAL_THUMB_EDGE, preview_from_jpeg};
use tauri::http::{Response, StatusCode, header};
use tauri::{AppHandle, Manager, UriSchemeResponder};

use crate::state::AppState;

/// How this platform spells the `media` scheme. Desktop webviews hand the
/// custom scheme to us directly; Android reroutes the `http://media.localhost`
/// alias through `shouldInterceptRequest`, so that is the only spelling a media
/// element may use there.
#[cfg(desktop)]
const MEDIA_ORIGIN: &str = "media://localhost";
#[cfg(mobile)]
const MEDIA_ORIGIN: &str = "http://media.localhost";

/// Address of one media route. `commands::catalog::media_source` is the only
/// caller: no component builds a transport URL itself.
pub(crate) fn media_address(route: &str, value: &str) -> String {
    format!("{MEDIA_ORIGIN}/{route}/{}", urlencoding::encode(value))
}

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

fn get_thumb(app: AppHandle, id: String, responder: UriSchemeResponder) {
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
        match core.browse.get_thumb_bytes(&id).await {
            Ok(bytes) => {
                let _ = cache.put(&id, &bytes);
                // Derive the blur-up placeholder from the JPEG we already
                // downloaded, so it never costs an extra request. Done
                // before responding so the mosaic lands on first paint.
                let mosaic = match core.browse.get_asset_thumb_hash(&id).await {
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
        }
    });
}

/// `media://<route>/<value>` - the one transport for every file the UI shows.
/// Routes are `thumb` (catalog thumbnail), `original` (catalog file) and
/// `local` (thumbnail of a file outside the catalog).
pub(crate) fn media(
    app: AppHandle,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let path = urlencoding::decode(request.uri().path().trim_start_matches('/'))
        .map(|v| v.to_string())
        .unwrap_or_default();
    let range = request
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let Some((route, value)) = path.split_once('/') else {
        responder.respond(not_found());
        return;
    };
    let (route, value) = (route.to_owned(), value.to_owned());
    match route.as_str() {
        "thumb" => get_thumb(app, value, responder),
        "original" => original(app, value, range, responder),
        "local" => get_local_thumb(app, value, responder),
        _ => responder.respond(not_found()),
    }
}

fn not_found() -> Response<Vec<u8>> {
    http_response(404, "text/plain", Vec::new(), None, "no-store")
}

/// `original` - the catalog file itself, with the webview's `Range` forwarded
/// to Cloudreve. `UriSchemeResponder` has no streaming form, so core caps one
/// response at 8 MiB instead of buffering a whole video.
fn original(app: AppHandle, id: String, range: Option<String>, responder: UriSchemeResponder) {
    tauri::async_runtime::spawn(async move {
        let core = {
            let state = app.state::<AppState>();
            Arc::clone(&state.core)
        };
        match core.browse.get_media_range(&id, range.as_deref()).await {
            Ok(media) => responder.respond(http_response(
                media.status,
                media
                    .content_type
                    .as_deref()
                    .unwrap_or("application/octet-stream"),
                media.body,
                media.content_range,
                "no-store",
            )),
            Err(error) => responder.respond(error_response(&error)),
        }
    });
}

/// Cache key that follows the file on disk: a rewrite bumps the mtime and
/// therefore the key, so a stale JPEG is never served.
#[cfg(desktop)]
fn local_thumb_key(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file://").unwrap_or(uri);
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

/// A media-library row is a `content://` uri, which cannot be stat'ed. The
/// adapter always answers for a row that exists, so the uri alone is the key;
/// a photo replaced in place keeps its uri, and its thumbnail with it.
#[cfg(mobile)]
fn local_thumb_key(uri: &str) -> Option<String> {
    Some(format!("local:{LOCAL_THUMB_EDGE}:{uri}"))
}

fn local_thumb_response(bytes: Vec<u8>) -> Response<Vec<u8>> {
    http_response(200, "image/jpeg", bytes, None, "public, max-age=86400")
}

/// `local` - thumbnails for files outside the Cloudreve catalog.
///
/// Streaming the original bytes here read multi-megabyte photos (and whole
/// videos) on the UI thread, which froze the window while a folder painted.
/// This answers with a cached 512px JPEG built by the local media port, so the
/// file itself is only ever read by the platform adapter.
fn get_local_thumb(app: AppHandle, raw: String, responder: UriSchemeResponder) {
    tauri::async_runtime::spawn(async move {
        let (core, cache) = {
            let state = app.state::<AppState>();
            (Arc::clone(&state.core), Arc::clone(&state.cache))
        };
        let Some(key) = local_thumb_key(&raw) else {
            responder.respond(not_found());
            return;
        };
        if let Some(bytes) = cache.get(&key) {
            responder.respond(local_thumb_response(bytes));
            return;
        }
        match core.browse.get_local_thumb(&raw, LOCAL_THUMB_EDGE).await {
            Ok(bytes) => {
                let _ = cache.put(&key, &bytes);
                responder.respond(local_thumb_response(bytes));
            }
            Err(_) => responder.respond(http_response(
                415,
                "text/plain",
                Vec::new(),
                None,
                "no-store",
            )),
        }
    });
}
