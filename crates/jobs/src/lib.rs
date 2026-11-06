//! A queue of background jobs, drained a few at a time.
//!
//! The crate knows nothing about what a job does. A caller hands it a kind, the
//! id of one item and the task working on that item - the same gesture as
//! [`tokio::spawn`] - and the queue answers with a [`Job`] on the board: queued,
//! running, how far it got, whether it ended.
//!
//! Two things make it more than a bare spawn. The number of tasks running at once
//! is one budget for the whole process, so nothing can exceed it by starting one
//! more batch; and every task runs under a kind and an id, so a whole kind can be
//! stopped with one call, and the same item never runs twice at the same time.
//! It is therefore the only place in the process allowed to spawn.

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

mod dto;

pub use dto::*;

/// How many finished jobs of one kind the board keeps, so a pass over thousands
/// of files does not grow the board for the rest of the process.
const KEEP_FINISHED: usize = 20;

/// A task, in the one shape the queue can hold while it waits its turn.
///
/// The error is a plain `String` on purpose: the queue only logs it and puts it
/// on the board, so it never links against an error crate or a business type.
type BoxTask = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>;

/// A handle on the queue, cheap to clone and shared by every caller.
#[derive(Clone)]
pub struct Jobs(Arc<Book>);

impl Jobs {
    /// At most `concurrency` tasks run at once, over every kind of work.
    pub fn new(concurrency: usize) -> Self {
        Self(Arc::new(Book {
            concurrency: concurrency.max(1),
            state: Mutex::new(State {
                board: Vec::new(),
                queue: VecDeque::new(),
                live: HashMap::new(),
            }),
            board: watch::Sender::new(Arc::new(Vec::new())),
            tracker: TaskTracker::new(),
        }))
    }

    /// Queue one item, and return at once.
    ///
    /// An item already queued or running is not queued twice: the item id says
    /// what the work is about, so a refresh pressed twice, a page reloaded or a
    /// pass offering the same file again is one job, not two.
    pub fn submit<F, Fut, E>(&self, key: JobKind, id: JobId, task: F)
    where
        F: FnOnce(Ctx) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: ToString + Send + 'static,
    {
        let cancel = CancellationToken::new();
        let ctx = Ctx {
            key,
            id: id.clone(),
            cancel: cancel.clone(),
            book: Arc::clone(&self.0),
        };
        let body: BoxTask =
            Box::pin(async move { task(ctx).await.map_err(|error| error.to_string()) });
        let board = {
            let mut state = self.0.lock();
            if state
                .board
                .iter()
                .any(|job| job.key == key && job.id == id && job.state.is_live())
            {
                return;
            }
            state
                .board
                .insert(0, Job::queued(id.clone(), key, now_ms()));
            state.queue.push_back(Pending {
                key,
                id,
                cancel,
                body,
            });
            state.snapshot()
        };
        self.0.publish(board);
        self.0.pump();
    }

    /// Stop everything of this kind: what is still waiting is dropped, what is
    /// running is asked to stop after the item it is on.
    pub fn cancel(&self, key: JobKind) {
        self.0.stop(|candidate, _| candidate == key);
    }

    /// Stop one item, which is what a row's own cancel button is.
    pub fn cancel_item(&self, key: JobKind, id: &JobId) {
        self.0
            .stop(|candidate, item| candidate == key && item == id);
    }

    /// Every job this process ran, newest first.
    pub fn list(&self) -> Vec<Job> {
        self.0.board.borrow().as_ref().clone()
    }

    /// The board as it changes, for a UI bridge.
    ///
    /// A [`watch`] is the right shape: it is a state, not an event stream, so a
    /// subscriber that wakes late still sees the current board and never a
    /// backlog of stale ones.
    pub fn watch(&self) -> watch::Receiver<Arc<Vec<Job>>> {
        self.0.board.subscribe()
    }
}

/// The queue itself, shared with every [`Ctx`].
struct Book {
    concurrency: usize,
    state: Mutex<State>,
    board: watch::Sender<Arc<Vec<Job>>>,
    tracker: TaskTracker,
}

impl Book {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap()
    }

    fn publish(&self, board: Vec<Job>) {
        self.board.send_replace(Arc::new(board));
    }

    /// Start as many waiting jobs as the budget allows.
    ///
    /// Called when something is queued and when something ends, which is the
    /// whole dispatcher: no loop of its own, nothing to shut down.
    fn pump(self: &Arc<Self>) {
        loop {
            let started = {
                let mut state = self.lock();
                if state.live.len() >= self.concurrency {
                    return;
                }
                let Some(pending) = state.queue.pop_front() else {
                    return;
                };
                let now = now_ms();
                if let Some(job) = state
                    .board
                    .iter_mut()
                    .find(|job| job.key == pending.key && job.id == pending.id)
                {
                    job.state = JobState::Running;
                    job.started_at = Some(now);
                }
                state
                    .live
                    .insert((pending.key, pending.id.clone()), pending.cancel.clone());
                (pending, state.snapshot())
            };
            let (pending, board) = started;
            self.publish(board);

            let book = Arc::clone(self);
            let Pending {
                key,
                id,
                cancel,
                body,
            } = pending;
            self.tracker.spawn(async move {
                let run = JobRun {
                    book: Arc::clone(&book),
                    key,
                    id: id.clone(),
                    cancel: cancel.clone(),
                    reported: false,
                };
                let outcome = body.await;
                run.report(outcome);
            });
        }
    }

    fn complete(
        self: &Arc<Self>,
        key: JobKind,
        id: &JobId,
        cancel: &CancellationToken,
        outcome: Result<(), String>,
    ) {
        let state = if cancel.is_cancelled() {
            JobState::Cancelled
        } else if outcome.is_err() {
            JobState::Failed
        } else {
            JobState::Done
        };
        if let Err(error) = &outcome {
            eprintln!("job {key} {id:?} failed: {error}");
        }
        let board = {
            let mut book = self.lock();
            book.live.remove(&(key, id.clone()));
            let now = now_ms();
            if let Some(job) = book
                .board
                .iter_mut()
                .find(|job| job.key == key && job.id == *id)
            {
                job.state = state;
                job.finished_at = Some(now);
            }
            prune(&mut book, key);
            book.snapshot()
        };
        self.publish(board);
        self.pump();
    }

    /// Drop the waiting items a predicate picks, and stop the running ones.
    fn stop(&self, hit: impl Fn(JobKind, &JobId) -> bool) {
        let board = {
            let mut book = self.lock();
            let mut dropped = Vec::new();
            book.queue.retain(|pending| {
                if hit(pending.key, &pending.id) {
                    dropped.push((pending.key, pending.id.clone()));
                    false
                } else {
                    true
                }
            });
            let now = now_ms();
            for (key, id) in &dropped {
                if let Some(job) = book
                    .board
                    .iter_mut()
                    .find(|job| job.key == *key && job.id == *id)
                {
                    job.state = JobState::Cancelled;
                    job.finished_at = Some(now);
                }
            }
            for ((key, id), token) in &book.live {
                if hit(*key, id) {
                    token.cancel();
                }
            }
            book.snapshot()
        };
        self.publish(board);
    }
}

/// What the queue holds behind one lock.
struct State {
    /// The board the UI sees, newest first.
    board: Vec<Job>,
    /// Work waiting for a slot, in submission order.
    queue: VecDeque<Pending>,
    /// Work running now, with the token that stops it.
    live: HashMap<(JobKind, JobId), CancellationToken>,
}

impl State {
    fn snapshot(&self) -> Vec<Job> {
        self.board.clone()
    }
}

struct Pending {
    key: JobKind,
    id: JobId,
    cancel: CancellationToken,
    body: BoxTask,
}

/// Keeps the last [`KEEP_FINISHED`] finished jobs of a kind on the board.
fn prune(state: &mut State, key: JobKind) {
    let mut kept = 0;
    state.board.retain(|job| {
        if job.key != key || job.state.is_live() {
            return true;
        }
        kept += 1;
        kept <= KEEP_FINISHED
    });
}

/// Reports the end of one job, whatever ends it - a plain return, an early
/// return, or a panic - so nothing is left running on the board forever.
struct JobRun {
    book: Arc<Book>,
    key: JobKind,
    id: JobId,
    cancel: CancellationToken,
    reported: bool,
}

impl JobRun {
    fn report(mut self, outcome: Result<(), String>) {
        self.reported = true;
        self.book
            .complete(self.key, &self.id, &self.cancel, outcome);
    }
}

impl Drop for JobRun {
    fn drop(&mut self) {
        if self.reported {
            return;
        }
        self.book.complete(
            self.key,
            &self.id,
            &self.cancel,
            Err("job task ended without reporting".to_owned()),
        );
    }
}

/// What a task is handed when it starts.
///
/// Cheap to clone: whatever a task runs beside itself keeps reporting to the
/// same job.
#[derive(Clone)]
pub struct Ctx {
    key: JobKind,
    id: JobId,
    cancel: CancellationToken,
    book: Arc<Book>,
}

impl Ctx {
    pub fn key(&self) -> JobKind {
        self.key
    }

    pub fn id(&self) -> &JobId {
        &self.id
    }

    /// The token of this job, for whatever needs to race a stop.
    pub fn cancel(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Count work done, in the unit this job chose.
    pub fn advance(&self, done: u64) {
        if done == 0 {
            return;
        }
        self.modify(|progress| progress.done += done);
    }

    /// Say how much work the item turned out to have, once it is known.
    pub fn total(&self, total: u64) {
        self.modify(|progress| progress.total = Some(total));
    }

    fn modify(&self, change: impl FnOnce(&mut Progress)) {
        let board = {
            let mut book = self.book.lock();
            let Some(job) = book
                .board
                .iter_mut()
                .find(|job| job.key == self.key && job.id == self.id)
            else {
                return;
            };
            change(&mut job.progress);
            book.snapshot()
        };
        self.book.publish(board);
    }

    /// Run something beside the job, like a writer that drains a channel.
    ///
    /// It has no id of its own: it is not on the board and cannot be stopped
    /// alone, and it ends when the handle is dropped or the task returns.
    pub fn spawn<F>(&self, task: F) -> JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.book.tracker.spawn(task)
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}
