//! The `local_albums`, `local_album_mapping` and `local_assets` tables,
//! generated from [`crate::sqlite::MIGRATIONS`].

use super::folder::schema::folders;

diesel::table! {
    local_albums (id) {
        id -> Text,
        uri -> Text,
        name -> Text,
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
