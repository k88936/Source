//! The upload in flight, as a subscription a page holds.
//!
//! A row is only interesting while its bytes are moving: a queued row waits its
//! turn with nothing to show, and a finished one is history. So the set is the
//! three states an attempt passes through - building the session, sending the
//! chunks, and writing the entry back - asked again by id whenever a commit
//! names one. That keeps the set bounded by the concurrency, not by the backlog.

use std::collections::HashSet;
use std::future::Future;

use source_subscription::{Change, Delta, Row, Value, View};

use super::QueuedUpload;
use crate::store::{UPLOAD_QUEUE, UPLOAD_TASK_ID, UploadQueueStore, UploadState};

impl Row for QueuedUpload {
    fn id(&self) -> String {
        self.id.0.clone()
    }
}

/// Whether a row is one whose bytes are moving right now.
fn in_flight(state: UploadState) -> bool {
    matches!(
        state,
        UploadState::CreatingSession | UploadState::Uploading | UploadState::Completing
    )
}

pub struct RunningUploads<C> {
    catalog: C,
}

impl<C> RunningUploads<C> {
    pub fn new(catalog: C) -> Self {
        Self { catalog }
    }

    /// The upload rows this commit touched.
    ///
    /// Both sides count: a row that leaves the set is only named by the state it
    /// had, and a deleted row only has the side it left.
    fn touched(changes: &[Change]) -> HashSet<String> {
        let mut touched = HashSet::new();
        for change in changes {
            if change.table != UPLOAD_QUEUE.name {
                continue;
            }
            for side in [change.before(UPLOAD_TASK_ID), change.after(UPLOAD_TASK_ID)] {
                if let Some(Value::Text(id)) = side {
                    touched.insert(id.clone());
                }
            }
        }
        touched
    }
}

impl<C> View for RunningUploads<C>
where
    C: UploadQueueStore + 'static,
{
    /// Every row is the whole set, so the view takes no parameters.
    type Query = ();
    type Row = QueuedUpload;
    type Change = Change;

    async fn snapshot(&self, _query: &Self::Query) -> Result<Vec<QueuedUpload>, String> {
        let rows = self
            .catalog
            .list_upload_tasks()
            .await
            .map_err(|e| e.view_message())?;
        Ok(rows
            .into_iter()
            .filter(|row| in_flight(row.state))
            .map(QueuedUpload::from)
            .collect())
    }

    fn delta(
        &self,
        _query: &Self::Query,
        changes: &[Change],
    ) -> impl Future<Output = Result<Delta<QueuedUpload>, String>> + Send {
        async move {
            let mut delta = Delta::new();
            for id in Self::touched(changes) {
                match self
                    .catalog
                    .get_upload_task(&id)
                    .await
                    .map_err(|e| e.view_message())?
                {
                    Some(row) if in_flight(row.state) => {
                        delta.inserts.push(QueuedUpload::from(row));
                    }
                    _ => delta.deletes.push(id),
                }
            }
            Ok(delta)
        }
    }
}
