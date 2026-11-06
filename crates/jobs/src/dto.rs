//! What a job is, as the queue and the UI see it.

use serde::Serialize;
use serde::ser::Serializer;
use ts_rs::TS;

/// The kind of work a job belongs to, chosen by whoever submits it.
///
/// A key is opaque: the queue only compares it, to stop everything of one kind
/// at once. What a name means stays with the caller, as a constant next to the
/// work it names, which is what keeps this crate free of the business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TS)]
#[ts(export, type = "string")]
pub struct JobKind(&'static str);

impl JobKind {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for JobKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Serialize for JobKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0)
    }
}

/// Which item a job works on: a directory, a file, whatever the caller counts.
///
/// The queue never parses it. It is the same id the caller uses for the item, so
/// the same item submitted twice while it is still queued or running is one job,
/// and a UI row can be matched to the job working on it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, TS)]
#[ts(export, type = "string")]
pub struct JobId(pub String);

impl From<String> for JobId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for JobId {
    fn from(id: &str) -> Self {
        Self(id.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum JobState {
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

impl JobState {
    /// Whether the item is still in the queue's hands.
    pub fn is_live(self) -> bool {
        matches!(self, JobState::Queued | JobState::Running)
    }
}

/// How far a job got, in whatever unit its submitter counts.
///
/// The queue never computes these numbers; it only carries them. For a listing
/// the unit is a page, for an upload a chunk - one job, one unit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Progress {
    pub done: u64,
    /// `None` while the job has not counted its work yet, which is the usual
    /// case: a listing does not know how many pages the server will return.
    pub total: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Job {
    pub id: JobId,
    pub key: JobKind,
    pub state: JobState,
    pub progress: Progress,
    pub submitted_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

impl Job {
    pub(crate) fn queued(id: JobId, key: JobKind, submitted_at: i64) -> Self {
        Self {
            id,
            key,
            state: JobState::Queued,
            progress: Progress::default(),
            submitted_at,
            started_at: None,
            finished_at: None,
        }
    }
}
