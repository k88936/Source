//! Row structs mirroring the `upload_sessions` and `upload_session_records`
//! tables, plus their conversions to the uploader payloads.

use cloudreve_sdk_api::models::explorer::{
    EncryptMetadata, EncryptionCipher, StoragePolicy, UploadCredential,
};
use cloudreve_sdk_uploader::{ChunkProgress, UploadSession};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use super::schema;
use super::values::policy_type_from_str;
use crate::store::CatalogError;

/// The stored session, flattened into columns.
///
/// Every field of [`UploadCredential`] the uploader reads has a column; `ak` and
/// `key_time` are never read by the uploader, and the rest of the storage policy
/// stays on the server, so neither is persisted.
#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::upload_sessions)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct UploadSessionRow {
    pub(crate) file_size: i64,
    pub(crate) chunk_size: i64,
    pub(crate) session_id: String,
    pub(crate) credential: String,
    pub(crate) upload_id: String,
    pub(crate) uri: String,
    pub(crate) callback_secret: String,
    pub(crate) expires_at: i64,
    pub(crate) complete_url: Option<String>,
    pub(crate) upload_policy: Option<String>,
    pub(crate) mime_type: Option<String>,
    pub(crate) policy_type: Option<String>,
    pub(crate) relay: Option<bool>,
    pub(crate) chunk_concurrency: Option<i32>,
    pub(crate) encrypt_key_plain_text: Option<String>,
    pub(crate) encrypt_iv: Option<String>,
}

impl UploadSessionRow {
    /// Rebuild the session, with `upload_urls` in chunk order.
    pub(crate) fn into_session(
        self,
        upload_urls: Vec<String>,
    ) -> Result<UploadSession, CatalogError> {
        let encrypt_metadata = self.encrypt_metadata()?;
        let storage_policy = self
            .policy_type
            .as_deref()
            .map(policy_type_from_str)
            .transpose()?
            .map(|policy_type| StoragePolicy {
                policy_type,
                relay: self.relay,
                chunk_concurrency: self.chunk_concurrency,
                ..StoragePolicy::default()
            });
        let credential = UploadCredential {
            session_id: self.session_id,
            expires: self.expires_at,
            chunk_size: self.chunk_size,
            upload_urls: (!upload_urls.is_empty()).then_some(upload_urls),
            credential: self.credential,
            upload_id: self.upload_id,
            callback_secret: self.callback_secret,
            ak: None,
            key_time: None,
            complete_url: self.complete_url,
            storage_policy,
            uri: self.uri,
            mime_type: self.mime_type,
            upload_policy: self.upload_policy,
            encrypt_metadata,
        };
        Ok(UploadSession::new(self.file_size as u64, credential))
    }

    fn encrypt_metadata(&self) -> Result<Option<EncryptMetadata>, CatalogError> {
        match (&self.encrypt_key_plain_text, &self.encrypt_iv) {
            (Some(key_plain_text), Some(iv)) => Ok(Some(EncryptMetadata {
                algorithm: EncryptionCipher::Aes256Ctr,
                key_plain_text: key_plain_text.clone(),
                iv: iv.clone(),
            })),
            (None, None) => Ok(None),
            _ => Err(CatalogError::DecodeFailed {
                cause: "上传会话的加密元数据不完整".to_string(),
            }),
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::upload_session_records)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct UploadSessionRecordRow {
    pub(crate) chunk_index: i32,
    pub(crate) loaded: i64,
    pub(crate) etag: Option<String>,
}

impl From<UploadSessionRecordRow> for ChunkProgress {
    fn from(row: UploadSessionRecordRow) -> Self {
        ChunkProgress::new(row.chunk_index as usize, row.loaded as u64, row.etag)
    }
}
