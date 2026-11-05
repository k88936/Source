//! The `runs` table, generated from [`crate::sqlite::MIGRATIONS`].

diesel::table! {
    runs (id) {
        id -> Text,
        /// The run key: a string the use case picked, compared but never interpreted here.
        key -> Text,
        state -> Text,
        total -> BigInt,
        done -> BigInt,
        failed -> BigInt,
        started_at -> BigInt,
        finished_at -> Nullable<BigInt>,
    }
}
