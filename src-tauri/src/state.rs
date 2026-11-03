//! Shared Tauri state.

use std::path::PathBuf;
use std::sync::Arc;

use source_catalog::SqliteCatalog;
use source_cloudreve::CloudreveService;
use source_core::AppCore;
use source_media::{FsLocalMedia, ThumbCache};

/// The composed dependency graph.
///
/// This alias is the wiring table: swapping an adapter is changing a type
/// argument here, and the compiler checks every use case against it. Nothing
/// else in the app may build an `AppCore` with a different set of ports.
pub(crate) type App = AppCore<SqliteCatalog, CloudreveService, FsLocalMedia>;

/// Everything a command needs: the core use cases plus the thumbnail cache.
pub(crate) struct AppState {
    pub(crate) core: Arc<App>,
    pub(crate) cache: Arc<ThumbCache>,
    pub(crate) app_data: PathBuf,
}
