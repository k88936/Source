//! Local media: the platform's own library (MediaStore on Android, the
//! filesystem on desktop dev builds), plus the byte-level helpers the app
//! needs around it.
//!
//! The port is synchronous on purpose: MediaStore and JNI calls are blocking,
//! and a `Result`-returning sync trait keeps the adapters free of an async
//! runtime. Remote bytes are not fetched here: that belongs to
//! [`crate::remote::RemoteStorage`].

use crate::error::AppError;

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

pub trait LocalMediaPort: Send + Sync {
    fn get_permission_state(&self) -> MediaPermissionState;
    fn request_permission(&self) -> MediaPermissionState;

    /// The directories one level below `parent`, so the user can pick one.
    ///
    /// `parent = None` starts wherever the platform thinks browsing should
    /// begin (the home directory on desktop). Sub-directories only: a local
    /// album is a directory, never a file.
    fn browse(&self, parent: Option<&str>) -> Result<LocalBrowsePage, AppError>;

    /// The direct media files of one local album, newest first.
    ///
    /// Never recursive: nested files belong to their own album if the user
    /// adds that directory too.
    fn list_assets(
        &self,
        album_uri: &str,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, AppError>;
    fn get_asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError>;
    /// How many assets [`Self::assets`] would report, without reading them.
    fn count_assets(&self, album_uri: &str) -> Result<u32, AppError>;

    /// Small JPEG preview for a grid tile.
    ///
    /// Each platform picks its cheapest source: MediaStore thumbnails on
    /// Android, decode or `ffmpeg` on desktop.
    fn get_thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError>;
}
