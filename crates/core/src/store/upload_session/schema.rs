//! The `upload_sessions` and `upload_session_records` tables, generated from
//! [`crate::sqlite::MIGRATIONS`].

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
