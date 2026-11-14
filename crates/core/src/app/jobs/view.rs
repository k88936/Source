//! The job list, as a subscription a page holds.
//!
//! The queue lives in memory, so there is nothing to observe: the queue reports
//! every update as a [`JobChange`], and this reads the row back out of the list.
//! A change therefore carries an identity and never a copy of the row, which is
//! also why the view keeps no copy of what it last sent.

use serde::Serialize;
use source_jobs::JobChangeOp;
use source_subscription::{Delta, Row, View};
use ts_rs::TS;

use super::{Job, JobChange, JobId, JobKind, JobState, Jobs, Progress};

/// One row of the job list, as a page reads it.
///
/// The queue's row plus the key a page's copy is kept under: ids are only unique
/// within a kind, so a page cannot key by `id` alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JobRow {
    pub id: String,
    pub kind: JobKind,
    pub job_id: String,
    pub state: JobState,
    pub progress: Progress,
    pub submitted_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

/// What a page's copy is keyed by.
fn row_id(kind: JobKind, id: &JobId) -> String {
    format!("{}:{}", kind.as_str(), id.0)
}

impl From<&Job> for JobRow {
    fn from(job: &Job) -> Self {
        Self {
            id: row_id(job.kind, &job.id),
            kind: job.kind,
            job_id: job.id.0.clone(),
            state: job.state,
            progress: job.progress,
            submitted_at: job.submitted_at,
            started_at: job.started_at,
            finished_at: job.finished_at,
        }
    }
}

impl Row for JobRow {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Every job this process ran, newest first.
pub struct JobList {
    jobs: Jobs,
}

impl JobList {
    pub fn new(jobs: Jobs) -> Self {
        Self { jobs }
    }
}

impl View for JobList {
    /// Every row is the whole list, so the view takes no parameters.
    type Query = ();
    type Row = JobRow;
    type Change = JobChange<JobKind>;

    async fn snapshot(&self, _query: &Self::Query) -> Result<Vec<JobRow>, String> {
        Ok(self.jobs.list().iter().map(JobRow::from).collect())
    }

    async fn delta(
        &self,
        _query: &Self::Query,
        changes: &[Self::Change],
    ) -> Result<Delta<JobRow>, String> {
        let current = self.jobs.list();
        let mut delta = Delta::new();
        for change in changes {
            match current
                .iter()
                .find(|job| job.kind == change.kind && job.id == change.id)
            {
                Some(job) if change.op == JobChangeOp::Upsert => {
                    delta.inserts.push(JobRow::from(job));
                }
                _ => delta.deletes.push(row_id(change.kind, &change.id)),
            }
        }
        Ok(delta)
    }
}
