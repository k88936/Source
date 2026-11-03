//! Media helpers: client-side EXIF fallback lives in `core`; this crate owns
//! the disk thumbnail cache, hashing and the desktop filesystem implementation
//! of the local-media port (the Android MediaStore implementation plugs into
//! the same port).

mod fs_local;
mod hash;
mod image_thumb;
mod net;
mod thumb_hash;
mod thumbs;

pub use fs_local::FsLocalMedia;
pub use hash::sha256_file;
pub use image_thumb::{LOCAL_THUMB_EDGE, jpeg_thumb_from_path, video_thumb_from_path};
pub use net::{fetch_bytes, fetch_range};
pub use thumb_hash::preview_from_jpeg;
pub use thumbs::{THUMB_BUDGET_BYTES, ThumbCache};
