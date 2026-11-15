//! The `local_albums`, `local_album_mapping` and `local_assets` tables,
//! generated from [`crate::sqlite::MIGRATIONS`].

use diesel::internal::table_macro::StaticQueryFragment;
use diesel::query_source::Column;
use source_subscription::Table;

use super::folder::schema::folders;

/// The key a client's copy of a local album is held by.
pub(crate) const LOCAL_ALBUM_ID: &str = local_albums::id::NAME;

/// The column a change of the mapping names its local album by.
pub(crate) const LOCAL_ALBUM_MAPPING_ALBUM: &str = local_album_mapping::local_album_id::NAME;

/// The local album a staged file belongs to.
pub(crate) const LOCAL_ASSET_ALBUM: &str = local_assets::album_id::NAME;

/// What the local album view reads out of this family: the album row itself, the
/// mapping that gives it a target, and the staged files whose count and mosaic
/// the tile shows.
pub(crate) const LOCAL_ALBUMS: Table = Table::new(
    <local_albums::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[LOCAL_ALBUM_ID],
);
pub(crate) const LOCAL_ALBUM_MAPPING: Table = Table::new(
    <local_album_mapping::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[LOCAL_ALBUM_MAPPING_ALBUM],
);
pub(crate) const LOCAL_ASSETS: Table = Table::new(
    <local_assets::table as StaticQueryFragment>::STATIC_COMPONENT.0,
    &[LOCAL_ASSET_ALBUM],
);

diesel::table! {
    local_albums (id) {
        id -> Text,
        uri -> Text,
        name -> Text,
        scanned_at -> Nullable<BigInt>,
    }
}

// The edge from a local album to the album it syncs into. `local_album_id` is
// the primary key, so a local album has exactly one target; the album side is
// not unique, so several local directories can sync into one album.
diesel::table! {
    local_album_mapping (local_album_id) {
        local_album_id -> Text,
        album_id -> Text,
    }
}

diesel::table! {
    local_assets (id) {
        id -> Text,
        album_id -> Nullable<Text>,
        name -> Text,
        mime -> Nullable<Text>,
        size -> Nullable<BigInt>,
        date_taken -> Nullable<BigInt>,
        uri -> Text,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        duration_ms -> Nullable<BigInt>,
        checksum -> Nullable<Text>,
        uploaded_asset_id -> Nullable<Text>,
    }
}

diesel::joinable!(local_album_mapping -> local_albums (local_album_id));
diesel::joinable!(local_album_mapping -> folders (album_id));
diesel::allow_tables_to_appear_in_same_query!(folders, local_album_mapping, local_albums);
