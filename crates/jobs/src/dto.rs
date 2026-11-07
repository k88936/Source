//! What a job is, as the queue and the UI see it.

use serde::Serialize;
use ts_rs::TS;

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
    /// Stopped by the queue's pause, still holding its place until resume.
    Paused,
    Done,
    Failed,
    Cancelled,
}

impl JobState {
    /// Whether the item is still in the queue's hands.
    pub fn is_live(self) -> bool {
        matches!(
            self,
            JobState::Queued | JobState::Running | JobState::Paused
        )
    }

    /// Whether the item will make progress without the queue being resumed.
    pub fn is_in_flight(self) -> bool {
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

/// One row on the board: what the caller submitted, plus what the queue did
/// with it.
///
/// `kind` is the caller's own vocabulary for the work - a tag a UI can pick
/// words for - and the queue never reads it. What the queue does is compare it,
/// which is how it stops a whole kind at once and why `K` must be a value, not
/// a payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Job<K> {
    pub id: JobId,
    pub kind: K,
    pub state: JobState,
    pub progress: Progress,
    pub submitted_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

impl<K> Job<K> {
    pub(crate) fn new(id: JobId, kind: K, state: JobState, submitted_at: i64) -> Self {
        Self {
            id,
            kind,
            state,
            progress: Progress::default(),
            submitted_at,
            started_at: None,
            finished_at: None,
        }
    }
}
