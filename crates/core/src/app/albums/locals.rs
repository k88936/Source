//! The local album view: the sources the sync page stacks beside an album.
//!
//! A local album is a registered directory plus the album it syncs into, and
//! nothing else about it is stored: its count and mosaic are the device's, asked
//! again every time because the catalog only knows which files were staged. So a
//! commit names the albums it touched, each of those is asked again, and one
//! that is gone is a delete.

use std::collections::HashSet;
use std::future::Future;

use source_subscription::{Change, Delta, Row, Value, View};

use super::LocalAlbumMetadata;
use crate::media::LocalMediaPort;
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
/// The count and the mosaic are asked of the device, not of the catalog, so a
/// failure here is the one thing a subscriber can be told about a tile it will
/// never see.
async fn local_album_metadata<M>(
    album: LocalAlbumRecord,
    device: &M,
) -> Result<LocalAlbumMetadata, String>
where
    M: LocalMediaPort,
{
    let asset_count = device
        .count_assets(&album.uri)
        .await
        .map_err(|cause| format!("local media: {cause:?}"))?;
    let covers = device
        .list_assets(&album.uri, None, 4)
        .await
        .map_err(|cause| format!("local media: {cause:?}"))?;
    Ok(LocalAlbumMetadata {
        id: album.id,
        uri: album.uri,
        name: album.name,
        album_id: AlbumId(album.album_id),
        album_name: album.album_name,
        asset_count,
        cover_uris: covers.items.into_iter().map(|asset| asset.uri).collect(),
    })
}

pub struct Locals<C, M> {
    catalog: C,
    device: M,
}

impl<C, M> Locals<C, M> {
    pub fn new(catalog: C, device: M) -> Self {
        Self { catalog, device }
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
        M: LocalMediaPort,
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
                    .push(local_album_metadata(album, &self.device).await?),
                None => delta.deletes.push(id),
            }
        }
        Ok(delta)
    }
}

impl<C, M> View for Locals<C, M>
where
    C: LocalStore + 'static,
    M: LocalMediaPort + 'static,
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
            rows.push(local_album_metadata(album, &self.device).await?);
        }
        Ok(rows)
    }

    /// One of three tables means this tile moved: `local_albums` (name, uri),
    /// `local_album_mapping` (the target), and `local_assets`. The last one does
    /// not feed the tile - the count and mosaic are the device's - but a scan
    /// that stages a file is the only signal that the directory gained one the
    /// snapshot predates, so it is the moment to ask again.
    fn delta(
        &self,
        _query: &Self::Query,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<LocalAlbumMetadata>, String>> + Send {
        self.changed(changes)
    }
}
