//! `CloudrevePort` implemented against `cloudreve-sdk-api`.
//!
//! These are thin mappings onto the SDK; the multi-step flows live in
//! [`auth`](super::auth) (login/restore) and [`upload`](super::upload).

use super::*;

use async_trait::async_trait;

#[async_trait]
impl CloudrevePort for CloudreveService {
    async fn site_info(&self, base_url: &str) -> Result<SiteInfo, AppError> {
        let base = base_url.trim_end_matches('/');
        let client = self.client_for(base);
        let cfg = client
            .get_site_config("basic")
            .await
            .map_err(map_api_error)?;
        let thumbnail_size = client
            .get_site_config("thumb")
            .await
            .ok()
            .and_then(|c| c.thumbnail_width)
            .filter(|v| *v > 0)
            .map(|v| v as u32);
        Ok(SiteInfo {
            name: cfg.title.unwrap_or_else(|| "Cloudreve".to_string()),
            thumbnail_size,
        })
    }

    async fn login(
        &self,
        base_url: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginOutcome, AppError> {
        self.authenticate(base_url, email, password).await
    }

    async fn login_2fa(
        &self,
        base_url: &str,
        session_id: &str,
        otp: &str,
    ) -> Result<SessionState, AppError> {
        self.authenticate_otp(base_url, session_id, otp).await
    }

    async fn restore(&self, base_url: &str) -> Result<Option<SessionState>, AppError> {
        self.restore_session(base_url).await
    }

    async fn logout(&self, _base_url: &str) -> Result<(), AppError> {
        if let Ok(client) = self.current_client() {
            client.clear_tokens().await;
        }
        if let Ok(mut guard) = self.active.lock() {
            *guard = None;
        }
        self.store.clear();
        Ok(())
    }

    async fn capacity(&self) -> Result<Capacity, AppError> {
        let client = self.current_client()?;
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
        let client = self.current_client()?;
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
        let client = self.current_client()?;
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

    async fn thumb_url(&self, uri: &str) -> Result<String, AppError> {
        let client = self.current_client()?;
        let base = self.current_base()?;
        client
            .get_file_thumb(uri, None)
            .await
            .map(|response| rewrite_loopback_url(&base, &response.url))
            .map_err(map_api_error)
    }

    async fn media_url(&self, uri: &str) -> Result<String, AppError> {
        let client = self.current_client()?;
        let response = client
            .get_file_url(&FileURLService {
                uris: vec![uri.to_string()],
                ..Default::default()
            })
            .await
            .map_err(map_api_error)?;
        let base = self.current_base()?;
        response
            .urls
            .first()
            .map(|u| rewrite_loopback_url(&base, &u.url))
            .ok_or_else(|| AppError::not_found("无法获取媒体地址"))
    }

    async fn client_taken_at(&self, uri: &str) -> Result<Option<i64>, AppError> {
        let url = self.media_url(uri).await?;
        let response = self
            .http
            .get(&url)
            .header(reqwest::header::RANGE, "bytes=0-262143")
            .send()
            .await
            .map_err(|e| AppError::network(format!("读取 EXIF 失败: {e}")))?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::network(format!("读取 EXIF 失败: {e}")))?;
        Ok(crate::exif::parse_exif(&bytes).taken_at)
    }

    async fn patch_metadata(
        &self,
        uri: &str,
        patches: &[(String, Option<String>)],
    ) -> Result<(), AppError> {
        let client = self.current_client()?;
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
        let client = self.current_client()?;
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
        let client = self.current_client()?;
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
