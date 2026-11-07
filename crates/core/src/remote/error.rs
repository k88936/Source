//! What the remote layer can fail with, and how one Cloudreve answer becomes it.
//!
//! Both halves of the remote layer talk to the same server, so the transport
//! failures - not signed in, session expired, unreachable, rejected - are one
//! shared vocabulary. Everything a single function can additionally tell apart
//! (a file is gone, a folder name is taken) stays on that function's own type,
//! because only its caller knows how to react to it.

use cloudreve_sdk_api::ApiError;
use cloudreve_sdk_uploader::UploadError;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A failure that belongs to the server or to the way we reached it.
///
/// This is the one place "not signed in / expired / unreachable / rejected" is
/// written: every adapter function either keeps the variant or renames it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum RemoteError {
    /// No live account: nobody signed in, or the session is gone.
    NotSignedIn,
    /// The server told us the session is no longer valid.
    SessionExpired { message: Option<String> },
    /// The request never reached a server that answered.
    Unreachable { cause: String },
    /// The server answered and refused the request.
    Rejected { status: i32, message: String },
    /// The answer was not something this client can read.
    Protocol { cause: String },
}

impl RemoteError {
    /// Whether the server refused the session the account row holds.
    ///
    /// A missing account is a call made before signing in, not an ended
    /// session, so it is deliberately not included.
    pub fn ends_session(&self) -> bool {
        matches!(
            self,
            RemoteError::SessionExpired { .. }
                | RemoteError::Rejected {
                    status: 401 | 403,
                    ..
                }
        )
    }
}

/// Turn one SDK answer into the shared vocabulary. One table, one place.
pub(crate) fn map_api_error(err: ApiError) -> RemoteError {
    match err {
        ApiError::LoginRequired(message) => RemoteError::SessionExpired {
            message: Some(message),
        },
        ApiError::AccessTokenExpired
        | ApiError::RefreshTokenExpired
        | ApiError::NoTokensAvailable => RemoteError::SessionExpired { message: None },
        ApiError::InvalidToken(cause) => RemoteError::SessionExpired {
            message: Some(cause),
        },
        ApiError::ApiError { code, message, .. } => {
            if matches!(code, 401 | 40020 | 40089) {
                RemoteError::SessionExpired {
                    message: Some(message),
                }
            } else {
                RemoteError::Rejected {
                    status: code,
                    message,
                }
            }
        }
        ApiError::LockConflict { message, .. } => RemoteError::Rejected {
            status: 40073,
            message,
        },
        ApiError::BatchError { message, .. } => RemoteError::Rejected {
            status: 40081,
            message,
        },
        ApiError::RequestError(cause) => RemoteError::Unreachable {
            cause: cause.to_string(),
        },
        ApiError::JsonError(cause) => RemoteError::Protocol {
            cause: cause.to_string(),
        },
        ApiError::InvalidUrl(cause) => RemoteError::Protocol {
            cause: cause.to_string(),
        },
        ApiError::Other(cause) => RemoteError::Protocol { cause },
        ApiError::SseNotUpgraded { code, message } => RemoteError::Rejected {
            status: code,
            message,
        },
        ApiError::SseStreamError(cause) => RemoteError::Protocol { cause },
    }
}

/// Probing a server before signing in: its name and thumbnail edge.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ProbeError {
    Unreachable { cause: String },
    /// The address answered, but not like a Cloudreve server.
    NotCloudreve { detail: String },
    Rejected { status: i32, message: String },
    Protocol { cause: String },
}

/// Entering a password or a one-time code.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum SignInError {
    Unreachable { cause: String },
    /// The password (or the account) was refused.
    CredentialsRejected { message: Option<String> },
    /// The one-time code was refused.
    CodeRejected { message: Option<String> },
    Rejected { status: i32, message: String },
    /// The answer was not something this client can read.
    Protocol { cause: String },
}

/// Restoring or dropping a stored session.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum SessionError {
    Unreachable { cause: String },
    Rejected { status: i32, message: String },
    Protocol { cause: String },
}

/// Reading the account quota.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum CapacityError {
    Unreachable { cause: String },
    Unauthorized,
    Rejected { status: i32, message: String },
    Protocol { cause: String },
}

/// Listing a folder's children, or reading the folder itself.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ListError {
    /// The folder was deleted, renamed or moved on the server.
    FolderGone { uri: String },
    Transport { cause: RemoteError },
}

/// Fetching the bytes (or the signed address) of one media file.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum MediaFetchError {
    /// The file was deleted, renamed or moved on the server.
    AssetGone { uri: String },
    Permission { uri: String },
    Transport { cause: RemoteError },
}

/// Patching metadata of one file.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum MetadataError {
    AssetGone { uri: String },
    Permission { uri: String },
    Transport { cause: RemoteError },
}

/// Creating one folder.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum CreateFolderError {
    /// The server refused the name itself.
    NameRejected,
    ParentGone { path: String },
    AlreadyExists { path: String },
    QuotaExceeded,
    Permission,
    Unreachable { cause: String },
    Protocol { cause: String },
}

/// Moving files into a folder.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum MoveError {
    SourceGone { uri: String },
    DestinationGone { uri: String },
    Conflict { uri: String },
    QuotaExceeded,
    Permission,
    Unreachable { cause: String },
    Protocol { cause: String },
}

/// Uploading one local file to the server.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum UploadUriError {
    /// The upload session record could not be written or read.
    Store { cause: crate::store::CatalogError },
    /// The local bytes could not be opened or read.
    SourceUnreadable { cause: String },
    /// The upload session is gone and has to start from zero.
    SessionExpired,
    /// A chunk was refused or the connection dropped mid-transfer.
    ChunkFailed { cause: String },
    QuotaExceeded,
    Conflict { path: String },
    Unreachable { cause: String },
    /// The attempt stopped between two chunks because the caller asked it to.
    Interrupted,
}

impl UploadUriError {
    /// One SDK upload failure, kept as the closest variant this layer knows.
    pub(crate) fn from_upload(error: UploadError) -> Self {
        match error {
            UploadError::Cancelled => UploadUriError::Interrupted,
            UploadError::SessionExpired => UploadUriError::SessionExpired,
            UploadError::FileReadError(cause) => UploadUriError::SourceUnreadable { cause },
            UploadError::ChunkUploadFailed { message, .. } => {
                UploadUriError::ChunkFailed { cause: message }
            }
            UploadError::HttpError(cause) | UploadError::Other(cause) => {
                UploadUriError::ChunkFailed { cause }
            }
            UploadError::ProviderError { provider, message } => UploadUriError::ChunkFailed {
                cause: format!("{provider}: {message}"),
            },
            UploadError::MaxRetriesExceeded { chunk_index, max_retries } => {
                UploadUriError::ChunkFailed {
                    cause: format!("chunk {chunk_index} after {max_retries} retries"),
                }
            }
            other => UploadUriError::ChunkFailed {
                cause: other.to_string(),
            },
        }
    }

    /// One SDK answer that arrived while an upload was being driven.
    pub(crate) fn from_api(error: ApiError) -> Self {
        UploadUriError::from_remote(map_api_error(error))
    }

    /// One failure of the signed-in client, before any SDK call was made.
    pub(crate) fn from_remote(cause: RemoteError) -> Self {
        match cause {
            RemoteError::NotSignedIn | RemoteError::SessionExpired { .. } => {
                UploadUriError::SessionExpired
            }
            RemoteError::Unreachable { cause } => UploadUriError::Unreachable { cause },
            RemoteError::Rejected { status, message } => match status {
                403 => UploadUriError::QuotaExceeded,
                404 => UploadUriError::SessionExpired,
                40004 | 40073 => UploadUriError::Conflict { path: message },
                _ => UploadUriError::ChunkFailed { cause: message },
            },
            RemoteError::Protocol { cause } => UploadUriError::ChunkFailed { cause },
        }
    }
}
