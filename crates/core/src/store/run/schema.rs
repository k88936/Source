//! The `runs` table, generated from [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    runs (id) {
        id -> Text,
        kind -> Text,
        state -> Text,
        total -> BigInt,
        done -> BigInt,
        failed -> BigInt,
        started_at -> BigInt,
        finished_at -> Nullable<BigInt>,
    }
}
