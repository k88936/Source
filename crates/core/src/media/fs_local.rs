//! Desktop filesystem local library. On Android the MediaStore adapter is used
//! instead so that partial grants and content URIs are handled by the platform.
//!
//! There is no configured root here: the user adds the directories they care
//! about one by one, and every directory they add is one local album whose
//! direct media files are its assets.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::mapping::{media_type, mime_for};
use crate::store::{LocalAssetId, MediaType};
use crate::time::system_millis;

use super::{
    LocalAsset, LocalAssetPage, LocalBrowsePage, LocalEntry, LocalMediaPort, MediaError,
    MediaPermissionState, PermissionGrant, jpeg_thumb_from_path, video_thumb_from_path,
};

/// The directory last read, kept so paging through it walks the filesystem once.
struct Listing {
    dir: PathBuf,
    /// Newest first, ties broken by name: a page resumes at the key of the file
    /// it stopped at, so the order has to be total and not just the mtime.
    assets: Vec<LocalAsset>,
}

#[derive(Clone, Default)]
pub struct FsLocalMedia {
    listing: Arc<Mutex<Option<Listing>>>,
}

impl FsLocalMedia {
    pub fn new() -> Self {
        Self::default()
    }

    /// One page of the directory, reading it only when the listing on hand is
    /// not the one this key came from.
    ///
    /// A page is asked for by the key the previous one ended on, so the list a
    /// page is sliced from has to be the list that key came from; a key with no
    /// listing behind it, and a first page, both start the walk over.
    fn page(
        &self,
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, MediaError> {
        let dir = PathBuf::from(album_uri);
        let limit = limit.clamp(1, 200) as usize;
        let mut slot = self.listing.lock().unwrap();
        if let Some(held) = slot.as_ref()
            && cursor.is_some()
            && held.dir == dir
        {
            return Ok(slice(held, cursor, limit));
        }
        let assets = direct_media(&dir)?;
        let held = Listing { dir, assets };
        let page = slice(&held, cursor, limit);
        *slot = Some(held);
        Ok(page)
    }
}

/// The key a page resumes at: the file it stopped on, as mtime and name.
fn page_key(asset: &LocalAsset) -> String {
    format!("{}:{}", asset.date_taken, asset.name)
}

fn parse_key(key: &str) -> Option<(i64, &str)> {
    let (date, name) = key.split_once(':')?;
    Some((date.parse().ok()?, name))
}

/// Where the list resumes: the first file that sorts after the key.
///
/// The list is newest first and names ascending inside one mtime, so "after"
/// is a smaller mtime, or the same one with a name further down.
fn past(asset: &LocalAsset, date: i64, name: &str) -> bool {
    asset.date_taken < date || (asset.date_taken == date && asset.name.as_str() > name)
}

fn slice(listing: &Listing, cursor: Option<&str>, limit: usize) -> LocalAssetPage {
    let start = match cursor.and_then(parse_key) {
        None => 0,
        Some((date, name)) => listing
            .assets
            .iter()
            .position(|asset| past(asset, date, name))
            .unwrap_or(listing.assets.len()),
    };
    let items: Vec<LocalAsset> = listing
        .assets
        .iter()
        .skip(start)
        .take(limit)
        .cloned()
        .collect();
    let next = if start + items.len() < listing.assets.len() {
        items.last().map(page_key)
    } else {
        None
    };
    LocalAssetPage { items, next }
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

/// The direct media files of `dir`, newest first, ties broken by name.
fn direct_media(dir: &Path) -> Result<Vec<LocalAsset>, MediaError> {
    let entries = fs::read_dir(dir).map_err(|error| MediaError::Platform {
        cause: format!("无法读取目录 {}: {error}", dir.display()),
    })?;
    let mut assets: Vec<LocalAsset> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_media_path(path))
        .filter_map(|path| local_asset_from_path(&path))
        .collect();
    assets.sort_by(|left, right| {
        right
            .date_taken
            .cmp(&left.date_taken)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(assets)
}

fn local_asset_from_path(path: &Path) -> Option<LocalAsset> {
    let meta = fs::metadata(path).ok()?;
    let name = path.file_name()?.to_str()?.to_string();
    let mime = mime_for(&name).map(str::to_owned);
    let date_taken = meta.modified().ok().map(system_millis).unwrap_or(0);
    Some(LocalAsset {
        id: LocalAssetId(path.to_string_lossy().to_string()),
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
        self.page(album_uri, cursor, limit)
    }

    async fn read_range(&self, uri: &str, offset: u64, len: u32) -> Result<Vec<u8>, MediaError> {
        use std::io::{Read, Seek, SeekFrom};

        let path = Path::new(uri.strip_prefix("file://").unwrap_or(uri));
        let mut file = fs::File::open(path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => MediaError::AssetGone {
                uri: uri.to_string(),
            },
            _ => MediaError::Platform {
                cause: format!("无法打开 {}: {error}", path.display()),
            },
        })?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|error| MediaError::Platform {
                cause: format!("无法定位 {}: {error}", path.display()),
            })?;
        let mut buffer = vec![0u8; len as usize];
        let mut filled = 0;
        while filled < buffer.len() {
            match file.read(&mut buffer[filled..]) {
                Ok(0) => break,
                Ok(read) => filled += read,
                Err(error) => {
                    return Err(MediaError::Platform {
                        cause: format!("无法读取 {}: {error}", path.display()),
                    });
                }
            }
        }
        buffer.truncate(filled);
        Ok(buffer)
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
