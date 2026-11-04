//! The Cloudreve side of the application, split into the two contracts the use
//! cases see and the adapters that implement them.
//!
//! [`auth`] signs in and remembers the credentials; [`storage`] is every file
//! operation, and reaches the signed-in client through the narrow
//! [`auth::CurrentAccount`] trait instead of the auth adapter itself. Uploads
//! use `cloudreve-sdk-uploader`, which handles local, slave, S3-like, OneDrive,
//! Qiniu and Upyun providers.

pub(crate) mod auth;
pub(crate) mod error;
pub(crate) mod storage;

pub use auth::{CloudreveAuth, CurrentAccount, RemoteAuth, SignInOutcome, SignedIn, SiteInfo};
pub(crate) use error::map_api_error;
pub use storage::{
    Capacity, CloudreveStorage, MediaRange, ProgressSink, RemoteEntry, RemoteEntryPage,
    RemoteStorage, UploadProgress, UploadRequest,
};

pub(crate) const USER_AGENT: &str = "Source/0.1";
