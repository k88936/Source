//! A local album: one directory on this device and the album it syncs into.

use crate::store::id_type;

// A file on this device, as the media adapter names it.
id_type!(LocalAssetId);

/// What the tile of a local album shows: the files it staged, newest first.
#[derive(Debug, Clone, Default)]
pub struct LocalAlbumTile {
    /// How many files of the directory are in the catalog.
    pub count: u32,
    /// Up to four of them, for the mosaic, as the device named them.
    pub cover_uris: Vec<String>,
}

/// A registered local album together with its target album.
#[derive(Debug, Clone)]
pub struct LocalAlbumRecord {
    /// The adapter's handle for the directory; also the album's identity here.
    pub id: String,
    pub uri: String,
    pub name: String,
    pub album_id: String,
    pub album_name: String,
}
