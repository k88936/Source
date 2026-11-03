//! Desktop filesystem local library. On Android the MediaStore adapter is used
//! instead so that partial grants and content URIs are handled by the platform.

use std::fs;
use std::path::{Path, PathBuf};

use source_core::dto::{
    AppError, LocalAlbum, LocalAsset, LocalAssetPage, MediaPermissionState, MediaType,
    PermissionGrant,
};
use source_core::mapping::{media_type, mime_for};
use source_core::ports::LocalMediaPort;
use source_core::time::system_millis;

use crate::{jpeg_thumb_from_path, video_thumb_from_path};

pub struct FsLocalMedia {
    roots: Vec<PathBuf>,
}

impl FsLocalMedia {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    pub fn default_roots() -> Vec<PathBuf> {
        if let Ok(root) = std::env::var("CLOUDREVE_ALBUM_LOCAL_ROOT")
            && !root.trim().is_empty()
        {
            return vec![PathBuf::from(root)];
        }
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        let mut roots = Vec::new();
        if let Some(home) = home {
            for candidate in ["Pictures", "DCIM", "Downloads"] {
                let path = home.join(candidate);
                if path.is_dir() {
                    roots.push(path);
                }
            }
        }
        roots
    }

    fn all_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for root in &self.roots {
            collect(root, &mut out, 0);
        }
        out.sort();
        out
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 6 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out, depth + 1);
        } else if is_media_path(&path) {
            out.push(path);
        }
    }
}

fn is_media_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    matches!(media_type(None, name), MediaType::Image | MediaType::Video)
}

fn media_count(dir: &Path) -> u32 {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| is_media_path(&e.path()))
                .count() as u32
        })
        .unwrap_or(0)
}

fn local_asset_from_path(path: &Path) -> Option<LocalAsset> {
    let meta = fs::metadata(path).ok()?;
    let name = path.file_name()?.to_str()?.to_string();
    let mime = mime_for(&name).map(str::to_owned);
    let date_taken = meta.modified().ok().map(system_millis).unwrap_or(0);
    Some(LocalAsset {
        id: path.to_string_lossy().to_string(),
        album_id: None,
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

impl LocalMediaPort for FsLocalMedia {
    fn permission_state(&self) -> MediaPermissionState {
        MediaPermissionState {
            granted: PermissionGrant::Full,
            can_request: false,
        }
    }

    fn request_permission(&self) -> MediaPermissionState {
        self.permission_state()
    }

    fn albums(&self) -> Result<Vec<LocalAlbum>, AppError> {
        let mut albums = Vec::new();
        for root in &self.roots {
            if !root.is_dir() {
                continue;
            }
            let name = root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("本机")
                .to_string();
            albums.push(LocalAlbum {
                id: format!("dir:{}", root.to_string_lossy()),
                name,
                asset_count: media_count(root),
            });
            let Ok(entries) = fs::read_dir(root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let count = media_count(&path);
                if count == 0 {
                    continue;
                }
                albums.push(LocalAlbum {
                    id: format!("dir:{}", path.to_string_lossy()),
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("相册")
                        .to_string(),
                    asset_count: count,
                });
            }
        }
        albums.sort_by_key(|a| std::cmp::Reverse(a.asset_count));
        Ok(albums)
    }

    fn assets(
        &self,
        album_id: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, AppError> {
        let dir = album_id
            .and_then(|id| id.strip_prefix("dir:"))
            .map(PathBuf::from);
        let mut files: Vec<PathBuf> = match &dir {
            Some(dir) => fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && is_media_path(p))
                        .collect()
                })
                .unwrap_or_default(),
            None => self.all_files(),
        };
        files.sort_by_key(|p| {
            std::cmp::Reverse(
                fs::metadata(p)
                    .and_then(|m| m.modified())
                    .map(system_millis)
                    .unwrap_or(0),
            )
        });
        let offset = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
        let limit = limit.clamp(1, 200) as usize;
        let items: Vec<LocalAsset> = files
            .iter()
            .skip(offset)
            .take(limit)
            .filter_map(|p| local_asset_from_path(p))
            .collect();
        let next = if offset + limit < files.len() {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(LocalAssetPage { items, next })
    }

    fn asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError> {
        let path = PathBuf::from(id);
        Ok(local_asset_from_path(&path).map(|mut asset| {
            asset.album_id = path
                .parent()
                .map(|p| format!("dir:{}", p.to_string_lossy()));
            asset
        }))
    }

    fn thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError> {
        let path = Path::new(id);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let preview = match media_type(None, name) {
            MediaType::Image => jpeg_thumb_from_path(path, edge),
            MediaType::Video => video_thumb_from_path(path, edge),
            MediaType::Other | MediaType::Audio => None,
        };
        preview.ok_or_else(|| AppError::not_found("无法生成缩略图"))
    }
}
