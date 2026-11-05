//! Run payloads: one background pass over a batch of work.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::id_type;

id_type!(RunId);

/// What a pass is about, chosen by the use case that opens it.
///
/// The store only compares keys; what a key means - which rows a pass visits,
/// which events it reports - stays in the use case that started it. A new kind
/// of pass is a new constant there, not a variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunKey(&'static str);

impl RunKey {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }
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
    pub id: RunId,
    pub state: RunState,
    /// Items the pass planned to visit, fixed when it started.
    pub total: u32,
    pub done: u32,
    pub failed: u32,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}
