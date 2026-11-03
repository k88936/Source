//! The `assets` table, generated from [`crate::sqlite::MIGRATIONS`].

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
