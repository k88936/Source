//! Background runs: the passes running in this process.
//!
//! A pass is only worth seeing while the process that started it is alive: a
//! process that died cannot still be busy, and its progress has nothing to
//! resume. The book is therefore in memory, and the newest pass under each key
//! survives a window reload.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::time::now_ms;

mod dto;

pub use dto::*;

/// Keeps two passes started in the same millisecond apart.
static RUN_SEQ: AtomicU64 = AtomicU64::new(0);

/// The passes of this process, the newest under each key.
#[derive(Clone, Default)]
pub struct Runs {
    runs: Arc<Mutex<HashMap<RunKey, RunHandle>>>,
}

impl Runs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a pass under `key`, and say whether this call is the one that
    /// opened it.
    ///
    /// A pass of the same key that is still running is returned as it is with
    /// `false`, so a second trigger never starts a second pass.
    pub fn begin(&self, key: RunKey, total: u32) -> (RunHandle, bool) {
        let mut runs = self.runs.lock().unwrap();
        if let Some(run) = runs.get(&key)
            && run.snapshot().state == RunState::Running
        {
            return (run.clone(), false);
        }
        let now = now_ms();
        let run = Run {
            id: RunId(format!(
                "{}-{}-{}",
                key.as_str(),
                now,
                RUN_SEQ.fetch_add(1, Ordering::Relaxed)
            )),
            state: RunState::Running,
            total,
            done: 0,
            failed: 0,
            started_at: now,
            finished_at: None,
        };
        let run = RunHandle::new(run);
        runs.insert(key, run.clone());
        (run, true)
    }

    /// The newest pass under one key, whatever state it is in.
    pub fn get(&self, key: RunKey) -> Option<Run> {
        self.runs.lock().unwrap().get(&key).map(RunHandle::snapshot)
    }
}

/// One open pass, shared by the book and the tracker reporting on it.
#[derive(Clone)]
pub struct RunHandle(Arc<Mutex<Run>>);

impl RunHandle {
    fn new(run: Run) -> Self {
        Self(Arc::new(Mutex::new(run)))
    }

    pub fn snapshot(&self) -> Run {
        self.0.lock().unwrap().clone()
    }

    /// Count visited items, and return the pass as it now stands.
    pub fn advance(&self, ok: u32, failed: u32) -> Run {
        let mut run = self.0.lock().unwrap();
        run.done += ok;
        run.failed += failed;
        run.clone()
    }

    /// Close the pass. A pass that is already closed is returned unchanged.
    pub fn finish(&self, state: RunState) -> Run {
        let mut run = self.0.lock().unwrap();
        if run.state == RunState::Running {
            run.state = state;
            run.finished_at = Some(now_ms());
        }
        run.clone()
    }
}
