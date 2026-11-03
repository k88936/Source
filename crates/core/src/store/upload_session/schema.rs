//! The `upload_sessions`, `upload_session_urls` and `upload_session_records`
//! tables, generated from [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    upload_sessions (key) {
        key -> Text,
        file_size -> BigInt,
        chunk_size -> BigInt,
        session_id -> Text,
        credential -> Text,
        upload_id -> Text,
        uri -> Text,
        callback_secret -> Text,
        expires_at -> BigInt,
        complete_url -> Nullable<Text>,
        upload_policy -> Nullable<Text>,
        mime_type -> Nullable<Text>,
        policy_type -> Nullable<Text>,
        relay -> Nullable<Bool>,
        chunk_concurrency -> Nullable<Integer>,
        encrypt_key_plain_text -> Nullable<Text>,
        encrypt_iv -> Nullable<Text>,
        updated_at -> BigInt,
    }
}

diesel::table! {
    upload_session_urls (key, chunk_index) {
        key -> Text,
        chunk_index -> Integer,
        url -> Text,
    }
}

diesel::table! {
    upload_session_records (key, chunk_index) {
        key -> Text,
        chunk_index -> Integer,
        loaded -> BigInt,
        etag -> Nullable<Text>,
    }
}
