//! Local media: the platform's own library (MediaStore on Android, the
//! filesystem on desktop dev builds), plus the byte-level helpers the app
//! needs around it.
//!
//! The port is async because a phone library is reached through a platform
//! bridge, and opening it on Android is a permission request the user has to
//! answer before a single row comes back. Remote bytes are not fetched here:
//! that belongs to [`crate::remote::RemoteStorage`].

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

mod dto;
mod fs_local;
mod hash;
mod image_thumb;
mod thumb_hash;
mod thumbs;

pub use dto::*;
pub use fs_local::FsLocalMedia;
pub use hash::sha256_file;
pub use image_thumb::{LOCAL_THUMB_EDGE, jpeg_thumb_from_path, video_thumb_from_path};
pub use thumb_hash::preview_from_jpeg;
pub use thumbs::{THUMB_BUDGET_BYTES, ThumbCache};

/// What the local media library can fail with.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum MediaError {
    /// The platform keeps the library behind a permission the user refused.
    PermissionDenied,
    /// The file behind this URI is no longer there.
    AssetGone { uri: String },
    /// The platform bridge itself failed.
    Platform { cause: String },
    /// A preview could not be produced from the file.
    DecodeFailed { uri: String },
}

#[async_trait]
pub trait LocalMediaPort: Send + Sync {
    async fn get_permission_state(&self) -> Result<MediaPermissionState, MediaError>;

    /// Ask the platform for library access, and report what it answered.
    ///
    /// On Android this is the system permission dialog: the answer is whatever
    /// the user tapped, including the partial grant of "select photos".
    async fn request_permission(&self) -> Result<MediaPermissionState, MediaError>;

    /// The directories one level below `parent`, so the user can pick one.
    ///
    /// `parent = None` starts wherever the platform thinks browsing should
    /// begin: the home directory on desktop, and the whole library - every
    /// bucket, one flat list - on Android, where asking also raises the
    /// permission dialog. Sub-directories only: a local album is a directory,
    /// never a file.
    async fn browse(&self, parent: Option<&str>) -> Result<LocalBrowsePage, MediaError>;

    /// The direct media files of one local album, newest first.
    ///
    /// Never recursive: nested files belong to their own album if the user
    /// adds that directory too.
    async fn list_assets(
        &self,
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, MediaError>;
    async fn get_asset(&self, id: &str) -> Result<Option<LocalAsset>, MediaError>;
    /// How many assets [`Self::list_assets`] would report, without reading them.
    async fn count_assets(&self, album_uri: &str) -> Result<u32, MediaError>;

    /// Small JPEG preview for a grid tile.
    ///
    /// Each platform picks its cheapest source: MediaStore thumbnails on
    /// Android, decode or `ffmpeg` on desktop.
    async fn get_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, MediaError>;
}
