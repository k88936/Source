//! The length of the library: how many assets it holds.
//!
//! A window carries the rows a page shows, so a length cannot be read off it: a
//! page holds the intervals it scrolled to, and the head of the timeline is not
//! the timeline. The count is its own view, read back whole instead of patched
//! from a change, because no single row says what the sum became.

use serde::{Deserialize, Serialize};
use source_subscription::{Change, Delta, Row, View};
use ts_rs::TS;

use crate::store::{ASSETS, AssetStore};

/// The one row the library's length is held under.
const LIBRARY: &str = "library";

/// How many assets the library holds.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssetTotal {
    pub id: String,
    pub total: u32,
}

impl Row for AssetTotal {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// The length of the library.
pub struct Counts<C> {
    catalog: C,
}

impl<C: AssetStore> Counts<C> {
    pub fn new(catalog: C) -> Self {
        Self { catalog }
    }

    async fn total(&self) -> Result<AssetTotal, String> {
        let total = self
            .catalog
            .asset_count(None)
            .await
            .map_err(|e| e.view_message())?;
        Ok(AssetTotal {
            id: LIBRARY.to_string(),
            total,
        })
    }
}

impl<C> View for Counts<C>
where
    C: AssetStore + 'static,
{
    /// The library is the whole scope, so there is nothing to name.
    type Query = ();
    type Row = AssetTotal;
    type Change = Change;

    async fn snapshot(&self, _query: &()) -> Result<Vec<AssetTotal>, String> {
        Ok(vec![self.total().await?])
    }

    /// The number is recomputed, not adjusted: an asset that changed kind without
    /// moving, or a row an upsert left as it was, cannot make it drift.
    async fn delta(&self, _query: &(), changes: &[Change]) -> Result<Delta<AssetTotal>, String> {
        let mut delta = Delta::new();
        if changes.iter().any(|change| change.table == ASSETS.name) {
            delta.inserts.push(self.total().await?);
        }
        Ok(delta)
    }
}
