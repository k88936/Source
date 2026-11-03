//! Domain types, ports and application use cases.
//!
//! Everything here is independent of Tauri, SQLite, HTTP and Android. The
//! adapter crates (`catalog`, `cloudreve`, `media`) implement the ports; the
//! Tauri crate wires them together. TypeScript bindings are generated from the
//! DTOs in [`dto`].

pub mod app;
pub mod dto;
pub mod exif;
pub mod mapping;
pub mod ports;
pub mod time;

pub use app::AppCore;
pub use dto::*;
pub use ports::{
    AssetRecord, CatalogPort, CloudrevePort, FolderRecord, FolderSyncState, LocalMediaPort,
    ProgressSink, RemoteEntry, RemoteEntryPage, SearchQuery, StoredJournal, StoredRecord,
    StoredServer, StoredSession, UploadProgress, UploadRequest, UploadSessionPort,
};
