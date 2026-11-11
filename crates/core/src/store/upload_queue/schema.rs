//! The `upload_queue` table, generated from [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    upload_queue (id) {
        id -> Text,
        local_asset_id -> Text,
        local_album_id -> Nullable<Text>,
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
