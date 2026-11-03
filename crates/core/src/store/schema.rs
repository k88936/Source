//! Diesel table definitions generated from [`crate::sqlite::MIGRATIONS`].
//!
//! Regenerate with `diesel print-schema` after adding a migration; the only
//! hand edits are the `Bool`/`BigInt` mappings, which `print-schema` renders as
//! `Bool`/`BigInt` for SQLite already.

diesel::table! {
    server (id) {
        id -> Text,
        base_url -> Text,
        user_id -> Text,
        display_name -> Nullable<Text>,
        created_at -> BigInt,
    }
}

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
    assets (id) {
        id -> Text,
        folder_id -> Text,
        uri -> Text,
        name -> Text,
        size -> BigInt,
        created_at -> Nullable<BigInt>,
        updated_at -> BigInt,
        remote_updated_at -> BigInt,
        taken_at -> BigInt,
        taken_at_source -> Text,
        media_type -> Text,
        mime -> Nullable<Text>,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        duration_ms -> Nullable<BigInt>,
        favorite -> Bool,
        exif_json -> Nullable<Text>,
        exif_parsed -> Bool,
        source -> Text,
        checksum -> Nullable<Text>,
        thumb_hash -> Nullable<Text>,
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

diesel::table! {
    upload_queue (id) {
        id -> Text,
        local_asset_id -> Text,
        dest_uri -> Text,
        dest_name -> Nullable<Text>,
        name -> Text,
        state -> Text,
        uploaded -> BigInt,
        total -> BigInt,
        speed -> BigInt,
        eta_seconds -> Nullable<BigInt>,
        attempts -> BigInt,
        error -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    app_state (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    upload_sessions (key) {
        key -> Text,
        snapshot -> Binary,
        version -> Integer,
        expires_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    upload_session_records (key, chunk_index) {
        key -> Text,
        chunk_index -> Integer,
        record -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    app_state,
    assets,
    folders,
    local_albums,
    local_assets,
    server,
    sync_state,
    upload_queue,
    upload_session_records,
    upload_sessions,
);
