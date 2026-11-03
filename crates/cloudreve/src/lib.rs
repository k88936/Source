//! Cloudreve v4 adapter.
//!
//! Auth uses the documented password / OTP endpoints directly so 2FA
//! challenges (and their `session_id`) survive the typed SDK layer. Everything
//! else is built on `cloudreve-sdk-api`; uploads use
//! `cloudreve-sdk-uploader` which handles local, slave, S3-like, OneDrive,
//! Qiniu and Upyun providers.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cloudreve_sdk_api::api::{ExplorerApi, SiteApi, UserApi};
use cloudreve_sdk_api::models::explorer::{
    CreateFileService, FileResponse, FileURLService, GetFileInfoService, ListFileService,
    MetadataPatch, MoveFileService, PatchMetadataService, UploadCredential,
};
use cloudreve_sdk_api::models::user::Token;
use cloudreve_sdk_api::{ApiError, Client, ClientConfig};
use cloudreve_sdk_uploader::{
    ChunkProgress, ProgressUpdate, Tracker, UploadError, UploadEvent, UploadParams, UploadSession,
    UploadSource, UploadStream, Uploader, UploaderConfig,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use source_core::dto::{
    AppError, Capacity, ErrorCode, LoginOutcome, SessionState, SiteInfo, UserInfo,
};
use source_core::ports::{
    CloudrevePort, ProgressSink, RemoteEntry, RemoteEntryPage, StoredSession, UploadProgress,
    UploadRequest, UploadSessionPort,
};

pub(crate) const USER_AGENT: &str = "Source/0.1";
pub(crate) const PAGE_SIZE: i32 = 100;

mod api;
mod auth;
mod credential;
mod journal;
mod pipeline;
mod port;
mod remote;
mod session;
mod upload;

pub use credential::{CredentialStore, FileCredentialStore};
pub use session::CloudreveService;

pub(crate) use api::*;
pub(crate) use journal::*;
pub(crate) use remote::*;
pub(crate) use session::*;
pub(crate) use upload::*;
