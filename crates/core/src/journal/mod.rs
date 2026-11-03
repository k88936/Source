//! Durable storage for resumable upload journals.

use async_trait::async_trait;
use cloudreve_sdk_uploader::{ChunkProgress, UploadSession};

use crate::error::AppError;

mod codec;
pub(crate) mod schema;
pub(crate) mod sqlite;

pub(crate) use codec::*;

/// Durable storage for resumable upload journals.
///
/// A journal is append-only: the header is written once, then one row per
/// finished chunk. Re-appending a chunk index must replace that row instead of
/// growing the journal, so a retried chunk cannot corrupt the replay.
///
/// Only journals this build can interpret are loaded: a header of another
/// format version, and chunks that no longer parse, are dropped instead of
/// being replayed on every retry.
#[async_trait]
pub trait UploadSessionPort: Send + Sync {
    /// Store the header of a new session under `key`, dropping the chunk rows
    /// of the session it replaces.
    async fn create_session(&self, key: &str, session: &UploadSession) -> Result<(), AppError>;
    /// Append one finished chunk, replacing the row of a chunk retried within
    /// the same session.
    async fn append_chunk(&self, key: &str, record: &ChunkProgress) -> Result<(), AppError>;
    /// Return the header and the chunks of a session that can be resumed.
    async fn load_session(
        &self,
        key: &str,
    ) -> Result<Option<(UploadSession, Vec<ChunkProgress>)>, AppError>;
    /// Remove the header and every chunk recorded under `key`.
    async fn remove_session(&self, key: &str) -> Result<(), AppError>;
}
