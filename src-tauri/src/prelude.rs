//! Imports shared by the adapter modules.

pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;

pub(crate) use tauri::{AppHandle, Emitter, State};

pub(crate) use source_core::dto::{
    AlbumPageParams, AlbumSummary, AppError, AppInfo, AssetDetail, AssetDetailParams, AssetSummary,
    CacheStats, Capacity, CatalogChangeKind, CatalogChangedEvent, EnqueueUploadsParams,
    ExportAssetsParams, ExportResult, FolderRef, IndexPhase, IndexStatus, ListRemoteFoldersParams,
    LocalAlbum, LocalAssetPage, LocalAssetPageParams, Login2faParams, LoginOutcome, LoginParams,
    MediaPermissionState, MoveAssetsParams, RemoteFolderPage, SearchParams, SessionState,
    SetAlbumCoverParams, SetAlbumDescriptionParams, SetFavoriteParams, SetLibraryRootParams,
    SetServerParams, SiteInfo, StartIndexParams, TimelinePage, TimelinePageParams, UploadState,
    UploadStateEvent, UploadTask, UploadTaskParams,
};

pub(crate) use crate::state::{App, AppState};
