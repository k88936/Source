//! The whole engine: domain types, use cases, and the adapters that connect
//! them to SQLite, Cloudreve and the platform media library.
//!
//! Each module holds one dependency direction end to end — its contract, the
//! payload types of that contract, and the code that implements it. Everything
//! here is independent of Tauri: the Tauri crate only wires these pieces
//! together and maps commands. TypeScript bindings are generated from those
//! payload types, wherever they are declared.

pub mod app;
pub mod credential;
pub mod error;
pub mod exif;
pub mod journal;
pub mod mapping;
pub mod media;
pub mod remote;
pub mod sqlite;
pub mod store;
pub mod time;

pub use app::{
    BrowseService, FoldersService, IndexService, SessionService, UploadOutcome, UploadsService,
};
pub use credential::CredentialStore;
pub use error::{AppError, ErrorCode};
pub use journal::UploadSessionPort;
pub use media::{
    FsLocalMedia, LOCAL_THUMB_EDGE, ThumbCache, fetch_bytes, fetch_range, preview_from_jpeg,
};
pub use remote::{
    CloudrevePort, CloudreveService, ProgressSink, RemoteEntry, RemoteEntryPage, UploadProgress,
    UploadRequest,
};
pub use sqlite::SqliteCatalog;
pub use store::{
    AlbumStore, AssetRecord, AssetStore, FolderRecord, FolderStore, FolderSyncState, SearchQuery,
    ServerStore, TimelineStore, UploadQueueStore,
};
