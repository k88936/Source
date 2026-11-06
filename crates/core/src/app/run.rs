//! Booking the progress of one pass while it runs.
//!
//! A run is opened when the pass starts, counted as items are visited, and
//! closed when the batch ends. Whoever listens hears every change through a
//! plain closure, so the use case never knows it is reporting to a window.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::store::{Run, RunHandle, RunState};

/// Reports a run as it changes.
pub type RunSink = Arc<dyn Fn(&Run) + Send + Sync>;

/// One open run, shared by the attempts visiting its items.
#[derive(Clone)]
pub struct RunTracker {
    run: RunHandle,
    sink: RunSink,
    cancel: CancellationToken,
}

impl RunTracker {
    pub fn new(run: RunHandle, sink: RunSink, cancel: CancellationToken) -> Self {
        Self { run, sink, cancel }
    }

    /// The token the batch checks, and that a stop cancels.
    pub fn get_cancel(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Report the run exactly as it was opened.
    pub fn announce(&self) {
        let run = self.run.snapshot();
        self.emit(&run);
    }

    /// Count one visited item: `ok` is whether it was fully handled.
    pub async fn step(&self, ok: bool) -> Result<(), AppError> {
        let (done, failed) = if ok { (1, 0) } else { (0, 1) };
        self.advance(done, failed).await
    }

    /// Count several items at once: a whole album's listing is one update, not
    /// one update per file.
    pub async fn advance(&self, done: u32, failed: u32) -> Result<(), AppError> {
        if done == 0 && failed == 0 {
            return Ok(());
        }
        let run = self.run.advance(done, failed);
        self.emit(&run);
        Ok(())
    }

    /// Close the pass now that every attempt has returned.
    ///
    /// A stopped pass is cancelled even if some items were counted. Otherwise it
    /// is only failed when there was work and none of it succeeded, so a pass
    /// with a few failures still reports as done with a failure count.
    pub async fn close(&self) -> Result<(), AppError> {
        let state = if self.cancel.is_cancelled() {
            RunState::Cancelled
        } else {
            let run = self.run.snapshot();
            if run.total > 0 && run.failed >= run.total {
                RunState::Failed
            } else {
                RunState::Done
            }
        };
        let run = self.run.finish(state);
        self.emit(&run);
        Ok(())
    }

    fn emit(&self, run: &Run) {
        (self.sink)(run);
    }
}
