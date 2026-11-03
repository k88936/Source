//! The Cloudreve side of the application: the remote contract the use cases
//! call, and the adapter that implements it.
//!
//! Auth uses the documented password / OTP endpoints directly so 2FA
//! challenges (and their `session_id`) survive the typed SDK layer. Everything
//! else is built on `cloudreve-sdk-api`; uploads use
//! `cloudreve-sdk-uploader`, which handles local, slave, S3-like, OneDrive,
//! Qiniu and Upyun providers.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cloudreve_sdk_api::api::{ExplorerApi, SiteApi, UserApi};
use cloudreve_sdk_api::models::explorer::{
    CreateFileService, FileResponse, FileURLService, GetFileInfoService, ListFileService,
    MetadataPatch, MoveFileService, PatchMetadataService,
};
use cloudreve_sdk_api::models::user::Token;
use cloudreve_sdk_api::{ApiError, Client, ClientConfig};
use cloudreve_sdk_uploader::{
    ChunkProgress, ProgressUpdate, Tracker, UploadError, UploadEvent, UploadParams, UploadSource,
    UploadStream, Uploader, UploaderConfig,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::app::session::{Capacity, LoginOutcome, SessionState, SiteInfo, UserInfo};
use crate::credential::CredentialStore;
use crate::error::{AppError, ErrorCode};
use crate::journal::UploadSessionPort;

mod api;
mod auth;
mod dto;
mod entries;
mod port;
mod session;
mod transfer;

pub use dto::*;
pub use session::CloudreveService;

pub(crate) use api::*;
pub(crate) use entries::*;
pub(crate) use session::*;

pub(crate) const USER_AGENT: &str = "Source/0.1";
pub(crate) const PAGE_SIZE: i32 = 100;

#[async_trait]
pub trait CloudrevePort: Send + Sync {
    async fn site_info(&self, base_url: &str) -> Result<SiteInfo, AppError>;

    async fn login(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginOutcome, AppError>;

    async fn login_2fa(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SessionState, AppError>;

    /// Restore a persisted session, validating it against `/user/me`.
    async fn restore(&self, base_url: &str) -> Result<Option<SessionState>, AppError>;

    async fn logout(&self, base_url: &str) -> Result<(), AppError>;

    async fn capacity(&self) -> Result<Capacity, AppError>;

    async fn list_entries(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteEntryPage, AppError>;

    async fn folder_info(&self, uri: &str) -> Result<RemoteEntry, AppError>;

    /// Signed (short-lived) URL for a Cloudreve thumbnail.
    async fn thumb_url(&self, uri: &str) -> Result<String, AppError>;

    /// Signed (short-lived) URL for the original media/stream.
    async fn media_url(&self, uri: &str) -> Result<String, AppError>;

    /// Download just the head of a remote file and parse its EXIF timestamp.
    /// Used only when Cloudreve did not extract `exif:taken_at` server-side.
    async fn client_taken_at(&self, uri: &str) -> Result<Option<i64>, AppError>;

    /// Patch key/value metadata. `None` removes the key.
    async fn patch_metadata(
        &self,
        uri: &str,
        patches: &[(String, Option<String>)],
    ) -> Result<(), AppError>;

    async fn create_folder(&self, parent_uri: &str, name: &str) -> Result<RemoteEntry, AppError>;

    /// Move files into `dest_uri`, returning their new URIs in input order.
    ///
    /// In Cloudreve an album is a folder, so "add to album" is a move rather
    /// than a copy, and the URI (which embeds the path) changes with it.
    async fn move_assets(&self, uris: &[String], dest_uri: &str) -> Result<Vec<String>, AppError>;

    /// Upload a local asset. `source_uri` is `file://...` on desktop or
    /// `content://...` on Android; adapters must stream it without copying to
    /// an app cache and must stop between chunks once `cancel` is raised.
    async fn upload_uri(&self, request: UploadRequest) -> Result<RemoteEntry, AppError>;
}
