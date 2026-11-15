//! The local album view: the sources the sync page stacks beside an album.
//!
//! A local album is a registered directory plus the album it syncs into, and
//! nothing else about it is stored. Its tile is the catalog's: a walk stages one
//! row per file it finds, so what the directory holds is already in the index
//! and a card is painted without reading the device. A commit names the albums
//! it touched, each of those is asked again, and one that is gone is a delete.

use std::collections::HashSet;
use std::future::Future;

use source_subscription::{Change, Delta, Row, Value, View};

use super::LocalAlbumMetadata;
use crate::store::{
    AlbumId, LOCAL_ALBUM_ID, LOCAL_ALBUM_MAPPING, LOCAL_ALBUM_MAPPING_ALBUM, LOCAL_ALBUMS,
    LOCAL_ASSET_ALBUM, LOCAL_ASSETS, LocalAlbumRecord, LocalStore,
};

impl Row for LocalAlbumMetadata {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// One registered local album, with the tile the graph shows for it.
///
/// The count and the mosaic are the staging rows of the directory, so this is
/// two indexed statements and not a walk: a directory of ten thousand files
/// answers as fast as an empty one, and it is the same number the scan writes.
async fn local_album_metadata<C>(
    catalog: &C,
    album: LocalAlbumRecord,
) -> Result<LocalAlbumMetadata, String>
where
    C: LocalStore,
{
    let tile = catalog
        .local_album_tile(&album.id)
        .await
        .map_err(|cause| cause.view_message())?;
    Ok(LocalAlbumMetadata {
        id: album.id,
        uri: album.uri,
        name: album.name,
        album_id: AlbumId(album.album_id),
        album_name: album.album_name,
        asset_count: tile.count,
        cover_uris: tile.cover_uris,
    })
}

pub struct Locals<C> {
    catalog: C,
}

impl<C> Locals<C> {
    pub fn new(catalog: C) -> Self {
        Self { catalog }
    }

    /// The column a change of `table` names its local album by, if the tile is
    /// made of that table.
    fn route(table: &str) -> Option<&'static str> {
        if table == LOCAL_ALBUMS.name {
            Some(LOCAL_ALBUM_ID)
        } else if table == LOCAL_ALBUM_MAPPING.name {
            Some(LOCAL_ALBUM_MAPPING_ALBUM)
        } else if table == LOCAL_ASSETS.name {
            Some(LOCAL_ASSET_ALBUM)
        } else {
            None
        }
    }

    /// The local albums this commit touched.
    ///
    /// Both sides count: a mapping that moves to another album belongs to the
    /// source it left as much as to the one it landed in, and a deleted row only
    /// has the side it left.
    fn touched(changes: &[Change]) -> HashSet<String> {
        let mut touched = HashSet::new();
        for change in changes {
            let Some(column) = Self::route(change.table) else {
                continue;
            };
            for side in [change.before(column), change.after(column)] {
                if let Some(Value::Text(id)) = side {
                    touched.insert(id.clone());
                }
            }
        }
        touched
    }

    /// What the touched local albums look like now.
    ///
    /// A row that cannot be read back is an album that is gone, or a directory
    /// that is no longer mapped anywhere; the client drops an id it does not
    /// hold either way.
    async fn changed(&self, changes: &[Change]) -> Result<Delta<LocalAlbumMetadata>, String>
    where
        C: LocalStore,
    {
        let touched = Self::touched(changes);
        let mut delta = Delta::new();
        for id in touched {
            match self
                .catalog
                .get_local_album(&id)
                .await
                .map_err(|e| e.view_message())?
            {
                Some(album) => delta
                    .inserts
                    .push(local_album_metadata(&self.catalog, album).await?),
                None => delta.deletes.push(id),
            }
        }
        Ok(delta)
    }
}

impl<C> View for Locals<C>
where
    C: LocalStore + 'static,
{
    /// The local albums take no parameters: a set of sources is the whole
    /// library of them.
    type Query = ();
    type Row = LocalAlbumMetadata;
    type Change = Change;

    async fn snapshot(&self, _query: &Self::Query) -> Result<Vec<LocalAlbumMetadata>, String> {
        let albums = self
            .catalog
            .list_local_albums()
            .await
            .map_err(|e| e.view_message())?;
        let mut rows = Vec::with_capacity(albums.len());
        for album in albums {
            rows.push(local_album_metadata(&self.catalog, album).await?);
        }
        Ok(rows)
    }

    /// One of three tables means this tile moved: `local_albums` (name, uri),
    /// `local_album_mapping` (the target), and `local_assets`, because the tile
    /// is the staged rows - a scan writes one per file it finds, which is the
    /// number the card counts.
    fn delta(
        &self,
        _query: &Self::Query,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<LocalAlbumMetadata>, String>> + Send {
        self.changed(changes)
    }
}
