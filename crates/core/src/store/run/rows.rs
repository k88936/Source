//! Row struct mirroring the `runs` table, plus its DTO conversion.

use diesel::prelude::*;

use super::dto::{Run, RunId};
use super::schema::runs;
use super::values::run_state_from_str;

/// The row's columns, in the order [`RunRow`] declares them.
///
/// Spelled out instead of `RunRow::as_select()` so the row never carries the
/// run key: reading a pass back needs only what the UI shows.
pub(crate) type RunColumns = (
    runs::id,
    runs::state,
    runs::total,
    runs::done,
    runs::failed,
    runs::started_at,
    runs::finished_at,
);

pub(crate) fn run_columns() -> RunColumns {
    (
        runs::id,
        runs::state,
        runs::total,
        runs::done,
        runs::failed,
        runs::started_at,
        runs::finished_at,
    )
}

#[derive(Queryable)]
pub(crate) struct RunRow {
    pub(crate) id: String,
    pub(crate) state: String,
    pub(crate) total: i64,
    pub(crate) done: i64,
    pub(crate) failed: i64,
    pub(crate) started_at: i64,
    pub(crate) finished_at: Option<i64>,
}

impl From<RunRow> for Run {
    fn from(row: RunRow) -> Self {
        Run {
            id: RunId(row.id),
            state: run_state_from_str(&row.state),
            total: row.total.max(0) as u32,
            done: row.done.max(0) as u32,
            failed: row.failed.max(0) as u32,
            started_at: row.started_at,
            finished_at: row.finished_at,
        }
    }
}
