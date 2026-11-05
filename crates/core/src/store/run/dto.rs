//! Run payloads: one background pass over a batch of work.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which kind of pass a run row describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunKind {
    /// Re-list every album from the server.
    Index,
    /// Push the files of the local albums into their albums.
    Sync,
}

/// Where a run is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunState {
    Running,
    Done,
    Failed,
    Cancelled,
    /// A previous process died with the run still marked as running.
    Interrupted,
}

/// One pass: how far it got, and whether it is still going.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Run {
    pub id: String,
    pub kind: RunKind,
    pub state: RunState,
    /// Items the pass planned to visit, fixed when it started.
    pub total: u32,
    pub done: u32,
    pub failed: u32,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}
