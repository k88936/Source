//! Local media: the platform's own library (MediaStore on Android, the
//! filesystem on desktop dev builds), plus the byte-level helpers the app
//! needs around it.
//!
//! The port is synchronous on purpose: MediaStore and JNI calls are blocking,
//! and a `Result`-returning sync trait keeps the adapters free of an async
//! runtime.

use crate::error::AppError;

mod dto;
mod fs_local;
mod hash;
mod image_thumb;
mod net;
mod thumb_hash;
mod thumbs;

pub use dto::*;
pub use fs_local::FsLocalMedia;
pub use hash::sha256_file;
pub use image_thumb::{LOCAL_THUMB_EDGE, jpeg_thumb_from_path, video_thumb_from_path};
pub use net::{fetch_bytes, fetch_range};
pub use thumb_hash::preview_from_jpeg;
pub use thumbs::{THUMB_BUDGET_BYTES, ThumbCache};

pub trait LocalMediaPort: Send + Sync {
    fn permission_state(&self) -> MediaPermissionState;
    fn request_permission(&self) -> MediaPermissionState;
    fn albums(&self) -> Result<Vec<LocalAlbum>, AppError>;
    fn assets(
        &self,
        album_id: Option<&str>,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<LocalAssetPage, AppError>;
    fn asset(&self, id: &str) -> Result<Option<LocalAsset>, AppError>;

    /// Small JPEG preview for a grid tile.
    ///
    /// Each platform picks its cheapest source: MediaStore thumbnails on
    /// Android, decode or `ffmpeg` on desktop.
    fn thumb(&self, id: &str, edge: u32) -> Result<Vec<u8>, AppError>;
}
