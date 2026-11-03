//! The `server` and `app_state` tables, generated from
//! [`crate::sqlite::MIGRATIONS`].

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
    app_state (key) {
        key -> Text,
        value -> Text,
    }
}
