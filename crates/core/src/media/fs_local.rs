//! Desktop filesystem local library. On Android the MediaStore adapter is used
//! instead so that partial grants and content URIs are handled by the platform.
//!
//! There is no configured root here: the user adds the directories they care
//! about one by one, and every directory they add is one local album whose
//! direct media files are its assets.

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::mapping::{media_type, mime_for};
use crate::store::{LocalAssetId, MediaType};
use crate::time::system_millis;

use super::{
    LocalAsset, LocalAssetPage, LocalBrowsePage, LocalEntry, LocalMediaPort, MediaError,
    MediaPermissionState, PermissionGrant, jpeg_thumb_from_path, video_thumb_from_path,
};

#[derive(Clone, Default)]
pub struct FsLocalMedia;

impl FsLocalMedia {
    pub fn new() -> Self {
        Self
    }
}

/// The directory browsing starts at.
fn home() -> Result<PathBuf, MediaError> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|error| MediaError::Platform {
            cause: format!("无法确定主目录: {error}"),
        })
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn entry(path: &Path) -> LocalEntry {
    LocalEntry {
        uri: path.to_string_lossy().to_string(),
        name: display_name(path),
        path: None,
    }
}

fn is_media_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    matches!(media_type(None, name), MediaType::Image | MediaType::Video)
}

/// The direct media files of `dir`, newest first.
fn direct_media(dir: &Path) -> Result<Vec<PathBuf>, MediaError> {
    let entries = fs::read_dir(dir).map_err(|error| MediaError::Platform {
        cause: format!("无法读取目录 {}: {error}", dir.display()),
    })?;
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_media_path(path))
        .collect();
    files.sort_by_key(|path| {
        std::cmp::Reverse(
            fs::metadata(path)
                .and_then(|meta| meta.modified())
                .map(system_millis)
                .unwrap_or(0),
        )
    });
    Ok(files)
}

fn local_asset_from_path(path: &Path) -> Option<LocalAsset> {
    let meta = fs::metadata(path).ok()?;
    let name = path.file_name()?.to_str()?.to_string();
    let mime = mime_for(&name).map(str::to_owned);
    let date_taken = meta.modified().ok().map(system_millis).unwrap_or(0);
    Some(LocalAsset {
        id: LocalAssetId(path.to_string_lossy().to_string()),
        album_id: path
            .parent()
            .map(|parent| parent.to_string_lossy().to_string()),
        name,
        mime,
        size: meta.len() as i64,
        date_taken,
        uri: format!("file://{}", path.to_string_lossy()),
        width: None,
        height: None,
        duration_ms: None,
    })
}

#[async_trait]
impl LocalMediaPort for FsLocalMedia {
    async fn get_permission_state(&self) -> Result<MediaPermissionState, MediaError> {
        Ok(MediaPermissionState {
            granted: PermissionGrant::Full,
            can_request: false,
        })
    }

    async fn request_permission(&self) -> Result<MediaPermissionState, MediaError> {
        self.get_permission_state().await
    }

    async fn browse(&self, parent: Option<&str>) -> Result<LocalBrowsePage, MediaError> {
        let dir = match parent {
            Some(uri) => PathBuf::from(uri),
            None => home()?,
        };
        let entries = fs::read_dir(&dir).map_err(|error| MediaError::Platform {
            cause: format!("无法读取目录 {}: {error}", dir.display()),
        })?;
        let mut entries: Vec<LocalEntry> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .map(|path| entry(&path))
            .collect();
        entries.sort_by_key(|entry| entry.name.to_lowercase());
        Ok(LocalBrowsePage {
            current: Some(entry(&dir)),
            parent: dir.parent().map(entry),
            entries,
        })
    }

    async fn list_assets(
        &self,
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, MediaError> {
        let files = direct_media(Path::new(album_uri))?;
        let offset = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
        let limit = limit.clamp(1, 200) as usize;
        let items: Vec<LocalAsset> = files
            .iter()
            .skip(offset)
            .take(limit)
            .filter_map(|path| local_asset_from_path(path))
            .collect();
        let next = if offset + limit < files.len() {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(LocalAssetPage { items, next })
    }

    async fn count_assets(&self, album_uri: &str) -> Result<u32, MediaError> {
        Ok(direct_media(Path::new(album_uri))?.len() as u32)
    }

    async fn get_asset(&self, id: &str) -> Result<Option<LocalAsset>, MediaError> {
        Ok(local_asset_from_path(Path::new(id)))
    }

    async fn get_thumb(&self, uri: &str, edge: u32) -> Result<Vec<u8>, MediaError> {
        // The media route carries the app-side uri, not the bare path.
        let path = Path::new(uri.strip_prefix("file://").unwrap_or(uri));
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let preview = match media_type(None, name) {
            MediaType::Image => jpeg_thumb_from_path(path, edge),
            MediaType::Video => video_thumb_from_path(path, edge),
            MediaType::Other | MediaType::Audio => None,
        };
        preview.ok_or_else(|| MediaError::DecodeFailed {
            uri: uri.to_string(),
        })
    }
}
