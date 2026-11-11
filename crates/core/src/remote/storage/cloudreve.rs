//! The Cloudreve storage adapter: one signed-in client plus the upload sessions.

use std::sync::Arc;

use cloudreve_sdk_api::Client;

use super::*;
use crate::remote::RemoteError;
use crate::remote::map_api_error;
use crate::store::UploadSessionPort;

/// Cloning is cheap: the account and the sessions are shared handles.
#[derive(Clone)]
pub struct CloudreveStorage {
    account: Arc<dyn CurrentAccount>,
    pub(crate) http: reqwest::Client,
    pub(crate) sessions: Arc<dyn UploadSessionPort>,
}

/// The server answered 404 for one particular file.
fn media_fetch(uri: &str, cause: RemoteError) -> MediaFetchError {
    match cause {
        RemoteError::Rejected { status: 404, .. } => MediaFetchError::AssetGone {
            uri: uri.to_string(),
        },
        RemoteError::Rejected {
            status: 401 | 403, ..
        } => MediaFetchError::Permission {
            uri: uri.to_string(),
        },
        cause => MediaFetchError::Transport { cause },
    }
}

/// The server answered 404 for one particular folder.
fn listing(uri: &str, cause: RemoteError) -> ListError {
    match cause {
        RemoteError::Rejected { status: 404, .. } => ListError::FolderGone {
            uri: uri.to_string(),
        },
        cause => ListError::Transport { cause },
    }
}

fn patching(uri: &str, cause: RemoteError) -> MetadataError {
    match cause {
        RemoteError::Rejected { status: 404, .. } => MetadataError::AssetGone {
            uri: uri.to_string(),
        },
        RemoteError::Rejected {
            status: 401 | 403, ..
        } => MetadataError::Permission {
            uri: uri.to_string(),
        },
        cause => MetadataError::Transport { cause },
    }
}

impl CloudreveStorage {
    pub fn new(account: Arc<dyn CurrentAccount>, sessions: Arc<dyn UploadSessionPort>) -> Self {
        Self {
            account,
            http: reqwest::Client::new(),
            sessions,
        }
    }

    /// One SDK answer, with the session ending exactly where it is discovered.
    ///
    /// Only a refusal of the stored session counts; a missing account is a
    /// call made before signing in and does not end anything.
    pub(crate) fn answer(&self, err: cloudreve_sdk_api::ApiError) -> RemoteError {
        let cause = map_api_error(err);
        if matches!(
            cause,
            RemoteError::SessionExpired { .. }
                | RemoteError::Rejected {
                    status: 401 | 403,
                    ..
                }
        ) {
            self.account.session_lost();
        }
        cause
    }

    pub(crate) fn client(&self) -> Result<Arc<Client>, RemoteError> {
        Ok(self.account.current()?.client)
    }

    pub(crate) fn base(&self) -> Result<String, RemoteError> {
        Ok(self.account.current()?.base_url)
    }

    async fn signed_thumb_url(&self, uri: &str) -> Result<String, MediaFetchError> {
        let client = self.client().map_err(|cause| media_fetch(uri, cause))?;
        let base = self.base().map_err(|cause| media_fetch(uri, cause))?;
        client
            .get_file_thumb(uri, None)
            .await
            .map(|response| rewrite_loopback_url(&base, &response.url))
            .map_err(|cause| media_fetch(uri, self.answer(cause)))
    }

    async fn signed_media_url(&self, uri: &str) -> Result<String, MediaFetchError> {
        let client = self.client().map_err(|cause| media_fetch(uri, cause))?;
        let response = client
            .get_file_url(&FileURLService {
                uris: vec![uri.to_string()],
                ..Default::default()
            })
            .await
            .map_err(|cause| media_fetch(uri, self.answer(cause)))?;
        let base = self.base().map_err(|cause| media_fetch(uri, cause))?;
        response
            .urls
            .first()
            .map(|u| rewrite_loopback_url(&base, &u.url))
            .ok_or(MediaFetchError::AssetGone {
                uri: uri.to_string(),
            })
    }

    /// Whole body of a signed URL, refusing anything the server did not
    /// answer with 2xx.
    async fn get_bytes(&self, url: &str, uri: &str) -> Result<Vec<u8>, MediaFetchError> {
        let response = self.http.get(url).send().await.map_err(|cause| {
            media_fetch(
                uri,
                RemoteError::Unreachable {
                    cause: cause.to_string(),
                },
            )
        })?;
        let status = response.status();
        if !status.is_success() {
            return Err(media_fetch(
                uri,
                RemoteError::Rejected {
                    status: i32::from(status.as_u16()),
                    message: format!("HTTP {status}"),
                },
            ));
        }
        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|cause| {
                media_fetch(
                    uri,
                    RemoteError::Unreachable {
                        cause: cause.to_string(),
                    },
                )
            })
    }

    /// One `Range` request, kept verbatim so the caller can replay the answer.
    async fn get_range(
        &self,
        url: &str,
        range: Option<&str>,
        uri: &str,
    ) -> Result<MediaRange, MediaFetchError> {
        let mut request = self.http.get(url);
        if let Some(range) = range {
            request = request.header(reqwest::header::RANGE, range);
        }
        let response = request.send().await.map_err(|cause| {
            media_fetch(
                uri,
                RemoteError::Unreachable {
                    cause: cause.to_string(),
                },
            )
        })?;
        let status = response.status().as_u16();
        let content_range = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let body = response
            .bytes()
            .await
            .map_err(|cause| {
                media_fetch(
                    uri,
                    RemoteError::Unreachable {
                        cause: cause.to_string(),
                    },
                )
            })?
            .to_vec();
        Ok(MediaRange {
            status,
            body,
            content_range,
            content_type,
        })
    }
}
use cloudreve_sdk_api::api::{ExplorerApi, UserApi};
use cloudreve_sdk_api::models::explorer::{
    CreateFileService, FileURLService, ListFileService, MetadataPatch, MoveFileService,
    PatchMetadataService,
};

#[async_trait]
impl RemoteStorage for CloudreveStorage {
    fn is_live(&self) -> bool {
        self.account.current().is_ok()
    }

    async fn get_capacity(&self) -> Result<Capacity, CapacityError> {
        let client = self.client().map_err(capacity)?;
        let cap = client
            .get_user_capacity()
            .await
            .map_err(|cause| capacity(self.answer(cause)))?;
        Ok(Capacity {
            total: cap.total,
            used: cap.used,
            storage_pack_total: cap.storage_pack_total,
        })
    }

    async fn list_entries(
        &self,
        uri: &str,
        page_token: Option<&str>,
    ) -> Result<RemoteEntryPage, ListError> {
        let client = self.client().map_err(|cause| listing(uri, cause))?;
        let params = ListFileService {
            uri: uri.to_string(),
            page: None,
            page_size: Some(PAGE_SIZE),
            order_by: Some("updated_at".to_string()),
            order_direction: Some("desc".to_string()),
            next_page_token: page_token.map(str::to_owned),
        };
        let response = client
            .list_files(&params)
            .await
            .map_err(|cause| listing(uri, self.answer(cause)))?;
        let entries = response
            .files
            .iter()
            .map(|f| to_entry(&join_uri(uri, &f.name), f))
            .collect::<Vec<_>>();
        let current = response.parent.as_ref().ok_or(ListError::Transport {
            cause: RemoteError::Protocol {
                cause: "服务器未返回当前目录".to_string(),
            },
        })?;
        Ok(RemoteEntryPage {
            entries,
            next: response.pagination.next_token,
            current: to_entry(uri.trim_end_matches('/'), current),
        })
    }

    /// A folder's own entry, taken from a listing: the listing's `parent` field
    /// describes the folder that was listed, the drive root included.
    async fn get_folder_info(&self, uri: &str) -> Result<RemoteEntry, ListError> {
        Ok(self.list_entries(uri, None).await?.current)
    }

    async fn get_thumb_bytes(&self, uri: &str) -> Result<Vec<u8>, MediaFetchError> {
        let url = self.signed_thumb_url(uri).await?;
        self.get_bytes(&url, uri).await
    }

    async fn get_media_bytes(&self, uri: &str) -> Result<Vec<u8>, MediaFetchError> {
        let url = self.signed_media_url(uri).await?;
        self.get_bytes(&url, uri).await
    }

    async fn get_media_range(
        &self,
        uri: &str,
        range: Option<&str>,
    ) -> Result<MediaRange, MediaFetchError> {
        let url = self.signed_media_url(uri).await?;
        self.get_range(&url, range, uri).await
    }

    async fn get_media_url(&self, uri: &str) -> Result<String, MediaFetchError> {
        self.signed_media_url(uri).await
    }

    async fn client_taken_at(&self, uri: &str) -> Result<Option<i64>, MediaFetchError> {
        let head = self.get_media_range(uri, Some("bytes=0-262143")).await?;
        if !head.is_success() {
            return Ok(None);
        }
        Ok(crate::exif::parse_exif(&head.body).taken_at)
    }

    async fn patch_metadata(
        &self,
        uri: &str,
        patches: &[(String, Option<String>)],
    ) -> Result<(), MetadataError> {
        let client = self.client().map_err(|cause| patching(uri, cause))?;
        let patches = patches
            .iter()
            .map(|(key, value)| MetadataPatch {
                key: key.clone(),
                value: value.clone(),
                remove: value.is_none().then_some(true),
            })
            .collect();
        client
            .patch_metadata(&PatchMetadataService {
                uris: vec![uri.to_string()],
                patches,
            })
            .await
            .map_err(|cause| patching(uri, self.answer(cause)))
    }

    async fn create_folder(
        &self,
        parent_uri: &str,
        name: &str,
    ) -> Result<RemoteEntry, CreateFolderError> {
        let client = self.client().map_err(create_folder)?;
        let uri = join_uri(parent_uri, name);
        let created = client
            .create_file(&CreateFileService {
                uri: uri.clone(),
                file_type: "folder".to_string(),
                err_on_conflict: Some(true),
                metadata: None,
            })
            .await
            .map_err(|cause| create_folder(self.answer(cause)))?;
        Ok(to_entry(&uri, &created))
    }

    async fn move_assets(&self, uris: &[String], dest_uri: &str) -> Result<Vec<String>, MoveError> {
        if uris.is_empty() {
            return Ok(Vec::new());
        }
        let client = self.client().map_err(moving)?;
        client
            .move_files(&MoveFileService {
                uris: uris.to_vec(),
                dst: dest_uri.to_string(),
                copy: None,
            })
            .await
            .map_err(|cause| moving(self.answer(cause)))?;
        // The remote path is embedded in the URI, so the new URI is the old
        // file name re-parented. The tail is already encoded, so it is reused
        // verbatim instead of being re-encoded.
        let dest = dest_uri.trim_end_matches('/');
        Ok(uris
            .iter()
            .map(|uri| {
                let name = uri.rsplit('/').next().unwrap_or_default();
                format!("{dest}/{name}")
            })
            .collect())
    }

    async fn upload_uri(&self, request: UploadRequest) -> Result<RemoteEntry, UploadUriError> {
        self.upload_asset(request).await
    }
}

fn capacity(cause: RemoteError) -> CapacityError {
    match cause {
        RemoteError::Unreachable { cause } => CapacityError::Unreachable { cause },
        RemoteError::NotSignedIn | RemoteError::SessionExpired { .. } => {
            CapacityError::Unauthorized
        }
        RemoteError::Rejected { status, message } => CapacityError::Rejected { status, message },
        RemoteError::Protocol { cause } => CapacityError::Protocol { cause },
    }
}

fn create_folder(cause: RemoteError) -> CreateFolderError {
    match cause {
        RemoteError::Unreachable { cause } => CreateFolderError::Unreachable { cause },
        RemoteError::NotSignedIn | RemoteError::SessionExpired { .. } => {
            CreateFolderError::Permission
        }
        RemoteError::Rejected { status, message } => match status {
            40004 | 40073 => CreateFolderError::AlreadyExists { path: message },
            40016 => CreateFolderError::ParentGone { path: message },
            40083 => CreateFolderError::QuotaExceeded,
            401 | 403 => CreateFolderError::Permission,
            _ => CreateFolderError::NameRejected,
        },
        RemoteError::Protocol { cause } => CreateFolderError::Protocol { cause },
    }
}

fn moving(cause: RemoteError) -> MoveError {
    match cause {
        RemoteError::Unreachable { cause } => MoveError::Unreachable { cause },
        RemoteError::NotSignedIn | RemoteError::SessionExpired { .. } => MoveError::Permission,
        RemoteError::Rejected { status, message } => match status {
            404 => MoveError::SourceGone { uri: message },
            40016 => MoveError::DestinationGone { uri: message },
            40004 | 40073 => MoveError::Conflict { uri: message },
            40083 => MoveError::QuotaExceeded,
            401 | 403 => MoveError::Permission,
            _ => MoveError::Conflict { uri: message },
        },
        RemoteError::Protocol { cause } => MoveError::Protocol { cause },
    }
}
