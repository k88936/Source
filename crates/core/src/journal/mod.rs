//! Durable storage for resumable upload journals.

use async_trait::async_trait;

use crate::error::AppError;

mod codec;
mod dto;
pub(crate) mod sqlite;

pub(crate) use codec::*;
pub use dto::*;

/// Durable storage for resumable upload journals.
///
/// A journal is append-only: the header is written once, then one row per
/// finished chunk. Re-appending a chunk index must replace that row instead of
/// growing the journal, so a retried chunk cannot corrupt the replay.
#[async_trait]
pub trait UploadSessionPort: Send + Sync {
    /// Store the header of a new session under `key`, dropping the chunk rows
    /// of the session it replaces.
    async fn create_session(&self, key: &str, session: &StoredSession) -> Result<(), AppError>;
    async fn append_chunk(&self, key: &str, index: u32, record: &[u8]) -> Result<(), AppError>;
    async fn load_session(&self, key: &str) -> Result<Option<StoredJournal>, AppError>;
    /// Remove the header and every chunk recorded under `key`.
    async fn remove_session(&self, key: &str) -> Result<(), AppError>;
}
