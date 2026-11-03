//! Application use cases. This is the only place that coordinates the
//! Cloudreve, Catalog and Media ports; the Tauri layer just maps arguments,
//! errors and events.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc;

use crate::dto::{
    AlbumPageParams, AlbumSummary, AppError, AssetDetail, AssetSummary, Capacity,
    EnqueueUploadsParams, ErrorCode, FolderRef, IndexPhase, IndexStatus, LocalAlbum,
    LocalAssetPageParams, LoginOutcome, MediaPermissionState, SearchParams, SessionState, SiteInfo,
    StartIndexParams, TimelinePage, TimelinePageParams, UploadProgressEvent, UploadState,
    UploadTask, UploadTaskId,
};
use crate::mapping::{COVER_KEY, DESCRIPTION_KEY, FAVORITE_KEY, asset_record, exif_fields};
use crate::ports::{
    AlbumStore, AssetStore, CloudrevePort, FolderRecord, FolderStore, LocalMediaPort, RemoteEntry,
    SearchQuery, ServerStore, StoredServer, TimelineStore, UploadProgress, UploadQueueStore,
    UploadRequest,
};

pub(crate) use crate::time::now_ms;

/// Upload progress is written to the catalog at most this often.
///
/// The uploader reports every chunk; persisting each report would turn a large
/// upload into thousands of writes that the UI never reads.
const PROGRESS_WRITE_INTERVAL: Duration = Duration::from_millis(400);

/// The application, generic over its ports.
///
/// The concrete graph is fixed at compile time by the composition root (see
/// `App` in the Tauri crate): swapping an adapter is changing a type argument,
/// and each use-case module declares the stores it is allowed to touch as
/// bounds on its `impl` block.
pub struct AppCore<C, R, M> {
    cloud: R,
    catalog: C,
    local: M,
    pending_server: Mutex<Option<String>>,
    session: Mutex<Option<SessionState>>,
    index: Mutex<IndexStatus>,
    index_running: AtomicBool,
    index_cancel: AtomicBool,
    /// Cancellation flags of the upload attempts currently running.
    upload_cancels: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl<C, R, M> AppCore<C, R, M> {
    pub fn new(cloud: R, catalog: C, local: M) -> Self {
        Self {
            cloud,
            catalog,
            local,
            pending_server: Mutex::new(None),
            session: Mutex::new(None),
            index: Mutex::new(IndexStatus {
                phase: IndexPhase::Idle,
                folders_done: 0,
                folders_total: 0,
                assets_seen: 0,
                started_at: None,
                error: None,
            }),
            index_running: AtomicBool::new(false),
            index_cancel: AtomicBool::new(false),
            upload_cancels: Mutex::new(HashMap::new()),
        }
    }
}

mod browse;
mod folders;
mod index;
mod session;
mod uploads;

pub use uploads::UploadOutcome;

fn folder_ref(entry: &RemoteEntry) -> FolderRef {
    FolderRef {
        id: entry.id.clone().into(),
        uri: entry.uri.clone(),
        name: entry.name.clone(),
        path: entry.path.clone(),
    }
}

fn is_media(asset: &AssetSummary) -> bool {
    matches!(
        asset.media_type,
        crate::dto::MediaType::Image | crate::dto::MediaType::Video
    )
}

fn clamp_limit(limit: u32) -> u32 {
    limit.clamp(1, 200)
}

fn normalize_base_url(raw: &str) -> Result<String, AppError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::internal("服务器地址不能为空"));
    }
    let candidate = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };
    let invalid = || AppError::internal("服务器地址格式不正确");
    let parsed = url::Url::parse(&candidate).map_err(|_| invalid())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err(invalid());
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}
