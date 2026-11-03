//! Imports shared by the adapter modules.

pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;

pub(crate) use tauri::{AppHandle, Emitter, State};

pub(crate) use source_core::app::browse::{
    AlbumPageParams, AssetDetail, AssetDetailParams, ExportAssetsParams, ExportResult,
    MoveAssetsParams, SearchParams, SetAlbumCoverParams, SetAlbumDescriptionParams,
    SetFavoriteParams, TimelinePageParams,
};
pub(crate) use source_core::app::folders::{
    ListRemoteFoldersParams, RemoteFolderPage, SetLibraryRootParams,
};
pub(crate) use source_core::app::index::{IndexPhase, IndexStatus, StartIndexParams};
pub(crate) use source_core::app::session::{
    AppInfo, Capacity, Login2faParams, LoginOutcome, LoginParams, SessionState, SetServerParams,
    SiteInfo,
};
pub(crate) use source_core::app::uploads::{
    EnqueueUploadsParams, UploadStateEvent, UploadTaskParams,
};
pub(crate) use source_core::app::{CatalogChangeKind, CatalogChangedEvent};
pub(crate) use source_core::error::AppError;
pub(crate) use source_core::media::{
    CacheStats, LocalAlbum, LocalAssetPage, LocalAssetPageParams, MediaPermissionState,
};
pub(crate) use source_core::store::{
    AlbumSummary, AssetSummary, FolderRef, TimelinePage, UploadState, UploadTask,
};

pub(crate) use crate::state::{App, AppState};
