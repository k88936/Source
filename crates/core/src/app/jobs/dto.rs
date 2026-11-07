//! What the app's jobs are, as the frontend sees them.

use serde::Serialize;
use ts_rs::TS;

/// What one row on the board is about, as the frontend sees it.
///
/// The queue only compares this tag, which is how it stops a whole kind at
/// once. The frontend picks the words, so this carries no name of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum JobKind {
    /// Re-reading one album's listing.
    Index,
    /// Scanning one local directory into its album.
    Sync,
    /// Uploading one file into an album.
    Upload,
}
