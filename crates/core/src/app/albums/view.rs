//! The album view: the grid the albums page shows.
//!
//! Every column of a tile is derived - a count, a mosaic, the outcome of the
//! last listing - so a row cannot be assembled from a changed one. A commit only
//! names the albums it touched, each of those is read back whole, and the result
//! is sent as it is.

use std::collections::HashSet;
use std::future::Future;

use source_subscription::{Change, Delta, Row, Value, View};

use crate::store::{
    ASSET_FOLDER_ID, ASSETS, AlbumMetadata, AlbumStore, FOLDER_ID, FOLDERS, SYNC_FOLDER_ID,
    SYNC_STATE,
};

impl Row for AlbumMetadata {
    fn id(&self) -> String {
        self.id.0.clone()
    }
}

/// Every album the user added, with its tile.
pub struct Albums<C> {
    catalog: C,
}

impl<C: AlbumStore> Albums<C> {
    pub fn new(catalog: C) -> Self {
        Self { catalog }
    }

    /// The column a change of `table` names its album by, if the tile is made of
    /// that table.
    fn route(table: &str) -> Option<&'static str> {
        if table == FOLDERS.name {
            Some(FOLDER_ID)
        } else if table == ASSETS.name {
            Some(ASSET_FOLDER_ID)
        } else if table == SYNC_STATE.name {
            Some(SYNC_FOLDER_ID)
        } else {
            None
        }
    }

    /// The albums this commit touched.
    ///
    /// Both sides count: an asset moved between albums belongs to the album it
    /// left as much as to the one it landed in, and a deleted asset only has the
    /// side it left.
    fn touched(changes: &[Change]) -> HashSet<String> {
        let mut touched = HashSet::new();
        for change in changes {
            let Some(column) = Self::route(change.table) else {
                continue;
            };
            for side in [change.old(column), change.new(column)] {
                if let Some(Value::Text(id)) = side {
                    touched.insert(id.clone());
                }
            }
        }
        touched
    }

    /// What the touched albums look like now.
    ///
    /// Each one is read back and sent, and one that is gone is a delete. Nothing
    /// is compared with what was sent before: a tile is a handful of bytes and a
    /// commit only touches the albums it named, so re-sending an unchanged tile
    /// costs less than holding the whole grid here would.
    async fn changed(&self, changes: &[Change]) -> Result<Delta<AlbumMetadata>, String> {
        let touched = Self::touched(changes);
        let mut delta = Delta::new();
        for id in touched {
            // A missing row is an album that is gone, or a folder that never was
            // one; the client drops an id it does not hold either way.
            match self
                .catalog
                .get_album(&id)
                .await
                .map_err(|e| e.view_message())?
            {
                Some(album) => delta.inserts.push(album),
                None => delta.deletes.push(id),
            }
        }
        Ok(delta)
    }
}

impl<C> View for Albums<C>
where
    C: AlbumStore + 'static,
{
    /// The album view takes no parameters: an album set is the whole library.
    type Query = ();
    type Row = AlbumMetadata;

    fn snapshot(
        &self,
        _query: &Self::Query,
    ) -> impl Future<Output = Result<Vec<AlbumMetadata>, String>> + Send {
        async move {
            self.catalog
                .list_albums()
                .await
                .map_err(|e| e.view_message())
        }
    }

    /// Everything the tile shows comes from these three tables: folders (name,
    /// cover, description), assets (count, mosaic) and sync_state (the last
    /// listing outcome).
    fn delta(
        &self,
        _query: &Self::Query,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<AlbumMetadata>, String>> + Send {
        self.changed(changes)
    }
}
