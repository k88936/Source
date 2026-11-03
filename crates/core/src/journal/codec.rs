//! How the header and the finished chunks are encoded into the opaque bytes the
//! catalog stores.

use std::sync::Arc;

use cloudreve_sdk_api::models::explorer::UploadCredential;
use cloudreve_sdk_uploader::{ChunkProgress, UploadSession};
use serde::{Deserialize, Serialize};

use super::*;

// ---------------------------------------------------------------------------
// Upload journal
// ---------------------------------------------------------------------------

/// Format version of the journal header kept in the catalog.
///
/// The catalog stores opaque bytes, so a header written by another format
/// version cannot be interpreted: it is dropped instead of being resumed.
pub(crate) const JOURNAL_VERSION: u32 = 2;

/// Cold half of a session: the fields `UploadSession::new` cannot recompute.
///
/// Written once, right after the session is created, and never updated again.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JournalSession {
    pub(crate) source_id: String,
    pub(crate) remote_uri: String,
    pub(crate) file_size: u64,
    pub(crate) credential: UploadCredential,
}

impl JournalSession {
    fn new(session: &UploadSession) -> Self {
        Self {
            source_id: session.source_id.clone(),
            remote_uri: session.remote_uri.clone(),
            file_size: session.file_size,
            credential: session.credential().clone(),
        }
    }

    pub(crate) fn session(&self) -> UploadSession {
        UploadSession::new(
            self.source_id.clone(),
            self.remote_uri.clone(),
            self.file_size,
            self.credential.clone(),
        )
    }

    fn encode(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    pub(crate) fn decode(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}

/// Durable upload journal backed by the catalog database.
///
/// The journal is append-only: the header above is written once when the
/// session is created, then one row per finished chunk. A retry replays those
/// rows instead of re-writing everything, so a long upload stays linear in
/// writes and never keeps the finished chunks in memory.
pub(crate) struct UploadJournal {
    pub(crate) sessions: Arc<dyn UploadSessionPort>,
}

impl UploadJournal {
    pub(crate) fn new(sessions: Arc<dyn UploadSessionPort>) -> Self {
        Self { sessions }
    }

    /// Read the journal stored under `key`.
    ///
    /// Anything this build cannot resume — a header of another version or one
    /// that no longer parses — is removed instead of being retried forever.
    /// Storage errors read as "no journal": losing a resume point must never
    /// fail the upload itself.
    pub(crate) async fn load(&self, key: &str) -> Option<(JournalSession, Vec<ChunkProgress>)> {
        let stored = self.sessions.load_session(key).await.ok().flatten()?;
        if stored.session.version != JOURNAL_VERSION {
            self.remove(key).await;
            return None;
        }
        let Some(header) = JournalSession::decode(&stored.session.bytes) else {
            self.remove(key).await;
            return None;
        };
        // Chunks that no longer parse only cost a re-upload of that chunk.
        let records = stored
            .records
            .iter()
            .filter_map(|row| serde_json::from_slice::<ChunkProgress>(&row.bytes).ok())
            .collect();
        Some((header, records))
    }

    /// Write the header of a new session, dropping the chunks of the session it
    /// replaces.
    pub(crate) async fn create(&self, key: &str, session: &UploadSession) -> Result<(), AppError> {
        let bytes = JournalSession::new(session)
            .encode()
            .ok_or_else(|| AppError::internal("无法序列化上传会话"))?;
        self.sessions
            .create_session(
                key,
                &StoredSession {
                    bytes,
                    version: JOURNAL_VERSION,
                    expires_at: session.expires_at,
                },
            )
            .await
    }

    /// Append one finished chunk, replacing the row of a chunk retried within
    /// the same session.
    pub(crate) async fn append(&self, key: &str, record: &ChunkProgress) -> Result<(), AppError> {
        let bytes = serde_json::to_vec(record)
            .map_err(|error| AppError::internal(format!("无法序列化上传分片记录: {error}")))?;
        self.sessions
            .append_chunk(key, record.index as u32, &bytes)
            .await
    }

    pub(crate) async fn remove(&self, key: &str) {
        let _ = self.sessions.remove_session(key).await;
    }
}
