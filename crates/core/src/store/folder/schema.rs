//! The `folders` and `sync_state` tables, generated from
//! [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    folders (id) {
        id -> Text,
        parent_id -> Nullable<Text>,
        name -> Text,
        uri -> Text,
        path -> Text,
        updated_at -> BigInt,
        indexed_at -> Nullable<BigInt>,
        is_album -> Bool,
        cover_asset_id -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    sync_state (folder_id) {
        folder_id -> Text,
        last_listed_at -> Nullable<BigInt>,
        last_page_token -> Nullable<Text>,
        remote_updated_at -> Nullable<BigInt>,
        status -> Text,
        last_error -> Nullable<Text>,
    }
}
