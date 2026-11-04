//! Application use cases, one service per use case.
//!
//! Each service owns the ports it needs and the state that belongs to it alone,
//! so the only place that knows the whole graph is the composition root in the
//! Tauri crate. Business rules stay here instead of leaking into the command
//! layer, and no service can reach a port it did not declare.

use std::time::Duration;

use crate::error::AppError;
use crate::mapping::{COVER_KEY, DESCRIPTION_KEY};
use crate::remote::RemoteEntry;
use crate::store::{FolderRecord, FolderRef, display_path};

pub mod account;
pub mod albums;
pub mod browse;
mod dto;
pub mod folders;
pub mod uploads;

pub use account::{AccountService, AccountState, LoginOutcome};
pub use albums::AlbumsService;
pub use browse::BrowseService;
pub use dto::*;
pub use folders::FoldersService;
pub use uploads::{UploadOutcome, UploadsService};

/// Upload progress is written to the catalog at most this often.
///
/// The uploader reports every chunk; persisting each report would turn a large
/// upload into thousands of writes that the UI never reads.
const PROGRESS_WRITE_INTERVAL: Duration = Duration::from_millis(400);

pub(crate) fn folder_ref(entry: &RemoteEntry) -> FolderRef {
    FolderRef {
        id: entry.id.clone().into(),
        uri: entry.uri.clone(),
        name: entry.name.clone(),
        path: display_path(&entry.path),
    }
}

pub(crate) fn folder_record(folder: &FolderRef, info: &RemoteEntry) -> FolderRecord {
    FolderRecord {
        id: folder.id.0.clone(),
        name: folder.name.clone(),
        uri: folder.uri.clone(),
        path: info.path.clone(),
        updated_at: info.updated_at,
        cover_asset_id: info.metadata.get(COVER_KEY).cloned(),
        description: info.metadata.get(DESCRIPTION_KEY).cloned(),
    }
}

pub(crate) fn clamp_limit(limit: u32) -> u32 {
    limit.clamp(1, 200)
}

pub(crate) fn normalize_base_url(raw: &str) -> Result<String, AppError> {
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
