//! Payloads of the upload journal.
//!
//! The bytes are opaque: only the uploader adapter knows how to encode or
//! interpret them, and only it decides whether a stored session can still be
//! resumed.

/// Header of one persisted upload session, written once per session.
///
/// The store never inspects the payload and never enforces expiry; the uploader
/// reports a session the server no longer knows.
#[derive(Debug, Clone)]
pub struct StoredSession {
    pub bytes: Vec<u8>,
    /// Header format version, so headers from another format are dropped.
    pub version: u32,
    /// Session expiry as a Unix timestamp in seconds.
    pub expires_at: i64,
}

/// One finished chunk of a stored session.
#[derive(Debug, Clone)]
pub struct StoredRecord {
    /// Chunk index; recording the same index again replaces the row.
    pub index: u32,
    pub bytes: Vec<u8>,
}

/// A session header together with the chunks that already finished.
#[derive(Debug, Clone)]
pub struct StoredJournal {
    pub session: StoredSession,
    pub records: Vec<StoredRecord>,
}
