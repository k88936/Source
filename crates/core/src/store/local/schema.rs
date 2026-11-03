//! The `local_albums` and `local_assets` tables, generated from
//! [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    local_albums (id) {
        id -> Text,
        name -> Text,
        asset_count -> Nullable<BigInt>,
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
