//! The Cloudreve storage adapter: one signed-in client plus the upload sessions.

use super::*;
use crate::remote::map_api_error;
use crate::store::UploadSessionPort;

/// Cloning is cheap: the account and the sessions are shared handles.
#[derive(Clone)]
pub struct CloudreveStorage {
    account: Arc<dyn CurrentAccount>,
    pub(crate) http: reqwest::Client,
    pub(crate) sessions: Arc<dyn UploadSessionPort>,
}

impl CloudreveStorage {
    pub fn new(account: Arc<dyn CurrentAccount>, sessions: Arc<dyn UploadSessionPort>) -> Self {
        Self {
            account,
            http: reqwest::Client::new(),
            sessions,
        }
    }

    pub(crate) fn client(&self) -> Result<Arc<Client>, AppError> {
        Ok(self.account.current()?.client)
    }

    pub(crate) fn base(&self) -> Result<String, AppError> {
        Ok(self.account.current()?.base_url)
    }

    async fn signed_thumb_url(&self, uri: &str) -> Result<String, AppError> {
        let client = self.client()?;
        let base = self.base()?;
        client
            .get_file_thumb(uri, None)
            .await
            .map(|response| rewrite_loopback_url(&base, &response.url))
            .map_err(map_api_error)
    }

    async fn signed_media_url(&self, uri: &str) -> Result<String, AppError> {
        let client = self.client()?;
        let response = client
            .get_file_url(&FileURLService {
                uris: vec![uri.to_string()],
                ..Default::default()
            })
            .await
            .map_err(map_api_error)?;
        let base = self.base()?;
        response
            .urls
            .first()
            .map(|u| rewrite_loopback_url(&base, &u.url))
            .ok_or_else(|| AppError::not_found("无法获取媒体地址"))
    }

    /// Whole body of a signed URL, refusing anything the server did not
    /// answer with 2xx.
    async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::network(format!("下载媒体失败: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            return Err(AppError::network(format!("下载媒体失败: HTTP {status}")));
        }
        response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|e| AppError::network(format!("读取媒体失败: {e}")))
    }

    /// One `Range` request, kept verbatim so the caller can replay the answer.
    async fn get_range(&self, url: &str, range: Option<&str>) -> Result<MediaRange, AppError> {
        let mut request = self.http.get(url);
        if let Some(range) = range {
            request = request.header(reqwest::header::RANGE, range);
        }
        let response = request
            .send()
            .await
            .map_err(|e| AppError::network(format!("请求媒体失败: {e}")))?;
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
            .map_err(|e| AppError::network(format!("读取媒体失败: {e}")))?
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
    CreateFileService, FileURLService, GetFileInfoService, ListFileService, MetadataPatch,
    MoveFileService, PatchMetadataService,
};

#[async_trait]
impl RemoteStorage for CloudreveStorage {
    async fn capacity(&self) -> Result<Capacity, AppError> {
        let client = self.client()?;
        let cap = client.get_user_capacity().await.map_err(map_api_error)?;
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
    ) -> Result<RemoteEntryPage, AppError> {
        let client = self.client()?;
        let params = ListFileService {
            uri: uri.to_string(),
            page: None,
            page_size: Some(PAGE_SIZE),
            order_by: Some("updated_at".to_string()),
            order_direction: Some("desc".to_string()),
            next_page_token: page_token.map(str::to_owned),
        };
        let response = client.list_files(&params).await.map_err(map_api_error)?;
        let entries = response
            .files
            .iter()
            .map(|f| to_entry(uri, f))
            .collect::<Vec<_>>();
        let parent = response.parent.as_ref().map(|p| {
            if is_root_uri(uri) {
                root_entry(uri, Some(p))
            } else {
                to_entry(uri, p)
            }
        });
        Ok(RemoteEntryPage {
            entries,
            next: response.pagination.next_token,
            parent,
        })
    }

    async fn folder_info(&self, uri: &str) -> Result<RemoteEntry, AppError> {
        if is_root_uri(uri) {
            let page = self.list_entries(uri, None).await?;
            return Ok(page.parent.unwrap_or_else(|| root_entry(uri, None)));
        }
        let client = self.client()?;
        let info = client
            .get_file_info(&GetFileInfoService {
                uri: Some(uri.to_string()),
                id: None,
                extended: Some(false),
                folder_summary: Some(false),
            })
            .await
            .map_err(map_api_error)?;
        let parent = uri.rsplit_once('/').map(|(p, _)| p).unwrap_or(uri);
        Ok(to_entry(parent, &info))
    }

    async fn thumb_bytes(&self, uri: &str) -> Result<Vec<u8>, AppError> {
        let url = self.signed_thumb_url(uri).await?;
        self.get_bytes(&url).await
    }

    async fn media_bytes(&self, uri: &str) -> Result<Vec<u8>, AppError> {
        let url = self.signed_media_url(uri).await?;
        self.get_bytes(&url).await
    }

    async fn media_range(&self, uri: &str, range: Option<&str>) -> Result<MediaRange, AppError> {
        let url = self.signed_media_url(uri).await?;
        self.get_range(&url, range).await
    }

    async fn client_taken_at(&self, uri: &str) -> Result<Option<i64>, AppError> {
        let head = self.media_range(uri, Some("bytes=0-262143")).await?;
        if !head.is_success() {
            return Ok(None);
        }
        Ok(crate::exif::parse_exif(&head.body).taken_at)
    }

    async fn patch_metadata(
        &self,
        uri: &str,
        patches: &[(String, Option<String>)],
    ) -> Result<(), AppError> {
        let client = self.client()?;
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
            .map_err(map_api_error)
    }

    async fn create_folder(&self, parent_uri: &str, name: &str) -> Result<RemoteEntry, AppError> {
        let client = self.client()?;
        let uri = join_uri(parent_uri, name);
        let created = client
            .create_file(&CreateFileService {
                uri: uri.clone(),
                file_type: "folder".to_string(),
                err_on_conflict: Some(true),
                metadata: None,
            })
            .await
            .map_err(map_api_error)?;
        Ok(to_entry(parent_uri, &created))
    }

    async fn move_assets(&self, uris: &[String], dest_uri: &str) -> Result<Vec<String>, AppError> {
        if uris.is_empty() {
            return Ok(Vec::new());
        }
        let client = self.client()?;
        client
            .move_files(&MoveFileService {
                uris: uris.to_vec(),
                dst: dest_uri.to_string(),
                copy: None,
            })
            .await
            .map_err(map_api_error)?;
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

    async fn upload_uri(&self, request: UploadRequest) -> Result<RemoteEntry, AppError> {
        self.upload_asset(request).await
    }
}
