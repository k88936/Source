//! Row struct mirroring the `runs` table, plus its DTO conversion.

use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use super::dto::Run;
use super::schema;
use super::values::{run_kind_from_str, run_state_from_str};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::runs)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct RunRow {
    pub(crate) id: String,
    pub(crate) kind: String,
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
            id: row.id,
            kind: run_kind_from_str(&row.kind),
            state: run_state_from_str(&row.state),
            total: row.total.max(0) as u32,
            done: row.done.max(0) as u32,
            failed: row.failed.max(0) as u32,
            started_at: row.started_at,
            finished_at: row.finished_at,
        }
    }
}
