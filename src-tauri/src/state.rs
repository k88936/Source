//! Shared Tauri state.

use std::path::PathBuf;
use std::sync::Arc;

use source_core::AppCore;
use source_media::ThumbCache;

/// Everything a command needs: the core use cases plus the thumbnail cache.
pub(crate) struct AppState {
    pub(crate) core: Arc<AppCore>,
    pub(crate) cache: Arc<ThumbCache>,
    pub(crate) app_data: PathBuf,
}
