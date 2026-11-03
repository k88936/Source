//! How the header and the finished chunks are encoded into the opaque bytes the
//! catalog stores.

use cloudreve_sdk_api::models::explorer::UploadCredential;
use cloudreve_sdk_uploader::UploadSession;
use serde::{Deserialize, Serialize};

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
    pub(crate) fn new(session: &UploadSession) -> Self {
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

    pub(crate) fn encode(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    pub(crate) fn decode(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}
