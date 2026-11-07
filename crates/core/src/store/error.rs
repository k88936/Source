//! What can fail inside the catalog, in terms the store itself understands.
//!
//! Every store contract returns this, so a use case can tell "the index is not
//! there" from "this statement failed" without reading a sentence.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A failure of the SQLite catalog.
///
/// `cause` is the leaf the adapter cannot name: a diesel error, a JSON error or
/// an OS error. It is kept for diagnosis and is never shown as the main text.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum CatalogError {
    /// The catalog file could not be opened, migrated, or reached.
    Unavailable { cause: String },
    /// A statement was rejected by SQLite.
    QueryFailed { cause: String },
    /// A row or a value could not be decoded.
    DecodeFailed { cause: String },
    /// A model could not be encoded for storage.
    EncodeFailed { cause: String },
}
