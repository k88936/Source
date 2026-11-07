//! A queue of background jobs, drained a few at a time.
//!
//! The crate knows nothing about what a job does. A caller hands it a kind, the
//! id of one item, a priority and the task working on that item - the same
//! gesture as [`tokio::spawn`] - and the queue answers with a [`Job`] on the
//! board: queued, running, how far it got, whether it ended.
//!
//! Two things make it more than a bare spawn. The number of tasks running at once
//! is one budget for the whole process, so nothing can exceed it by starting one
//! more batch; and every task runs under a kind and an id, so a whole kind can be
//! stopped with one call, and the same item never runs twice at the same time.
//! It is therefore the only place in the process allowed to spawn.
//!
//! Pausing stops new normal work and asks running normal work to park. A parked
//! job keeps its place and its progress, and resume runs it again from the
//! factory the caller submitted. Real-time work ignores the pause, but still
//! shares the same concurrency budget.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

mod dto;

pub use dto::*;

/// How many finished jobs of one kind the board keeps, so a pass over thousands
/// of files does not grow the board for the rest of the process.
const KEEP_FINISHED: usize = 20;

/// How often a parked task and the upload timer look at the pause flag.
const PAUSE_POLL: Duration = Duration::from_millis(25);

/// How one run of a task ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobOutcome {
    /// The task finished what the queue gave it.
    Done,
    /// The queue paused, and the task stopped at a safe point.
    ///
    /// The queue puts it back without ending it, so resume runs it again from
    /// its factory.
    Paused,
}

/// What one run returned, and whether the queue's pause stopped it.
#[derive(Debug)]
pub struct Ran<T> {
    pub value: T,
    pub paused: bool,
}

/// Where a job sits when the queue has more work than slots.
///
/// A lower `Normal` number runs first. `RealTime` is work the user just asked
/// for: it ignores the pause and runs before every `Normal` job, but still
/// shares the same concurrency budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobPriority {
    Normal(u32),
    RealTime,
}

impl JobPriority {
    fn is_pausable(self) -> bool {
        matches!(self, JobPriority::Normal(_))
    }
}

impl Ord for JobPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (JobPriority::RealTime, JobPriority::RealTime) => std::cmp::Ordering::Equal,
            (JobPriority::RealTime, JobPriority::Normal(_)) => std::cmp::Ordering::Less,
            (JobPriority::Normal(_), JobPriority::RealTime) => std::cmp::Ordering::Greater,
            (JobPriority::Normal(left), JobPriority::Normal(right)) => left.cmp(right),
        }
    }
}

impl PartialOrd for JobPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A task, in the one shape the queue can hold while it waits its turn.
///
/// The error is a plain `String` on purpose: the queue only logs it and puts it
/// on the board, so it never links against an error crate or a business type.
type BoxTask = Pin<Box<dyn Future<Output = Result<JobOutcome, String>> + Send + 'static>>;

/// A task factory: the queue can run the same item again after a pause.
type TaskFactory = Arc<dyn Fn(Ctx) -> BoxTask + Send + Sync + 'static>;

type LiveKey = (JobKind, JobId);

/// A handle on the queue, cheap to clone and shared by every caller.
#[derive(Clone)]
pub struct Jobs(Arc<Book>);

impl Jobs {
    /// At most `concurrency` tasks run at once, over every kind of work.
    pub fn new(concurrency: usize) -> Self {
        Self(Arc::new(Book {
            concurrency: concurrency.max(1),
            paused: AtomicBool::new(false),
            state: Mutex::new(State {
                board: Vec::new(),
                queue: Vec::new(),
                live: HashMap::new(),
            }),
            board: watch::Sender::new(Arc::new(Vec::new())),
            tracker: TaskTracker::new(),
        }))
    }

    /// Queue one item, and return at once.
    ///
    /// An item already queued, paused or running is not queued twice: the item
    /// id says what the work is about, so a refresh pressed twice, a page
    /// reloaded or a pass offering the same file again is one job, not two.
    /// Submitting it with a higher priority upgrades the waiting entry instead
    /// of being dropped by that check.
    pub fn submit<F, Fut, E>(&self, key: JobKind, id: JobId, priority: JobPriority, task: F)
    where
        F: Fn(Ctx) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<JobOutcome, E>> + Send + 'static,
        E: std::fmt::Debug + Send + 'static,
    {
        let task = Arc::new(task);
        let factory: TaskFactory = Arc::new(move |ctx| {
            let task = Arc::clone(&task);
            Box::pin(async move { task(ctx).await.map_err(|error| format!("{error:?}")) })
        });
        let board = {
            let mut state = self.0.lock();
            if state
                .board
                .iter()
                .any(|job| job.key == key && job.id == id && job.state.is_live())
            {
                let mut upgraded = false;
                if let Some(pending) = state
                    .queue
                    .iter_mut()
                    .find(|pending| pending.key == key && pending.id == id)
                    && priority < pending.priority
                {
                    pending.priority = priority;
                    upgraded = true;
                }
                if !upgraded {
                    return;
                }
                state.queue.sort_by_key(|pending| pending.priority);
                if priority == JobPriority::RealTime
                    && let Some(job) = state
                        .board
                        .iter_mut()
                        .find(|job| job.key == key && job.id == id)
                    && job.state == JobState::Paused
                {
                    job.state = JobState::Queued;
                }
                state.snapshot()
            } else {
                let job_state = self.0.job_state(priority);
                state
                    .board
                    .insert(0, Job::new(id.clone(), key, job_state, now_ms()));
                state.queue.push(Pending {
                    key,
                    id,
                    priority,
                    user_cancel: CancellationToken::new(),
                    factory,
                });
                state.queue.sort_by_key(|pending| pending.priority);
                state.snapshot()
            }
        };
        self.0.publish(board);
        self.0.pump();
    }

    /// Run `tick` every `interval`, starting at once.
    ///
    /// A timer is not a job: it has no id, it is not on the board and it never
    /// holds one of the `concurrency` slots. That is what a queue needs to look
    /// at its own backlog on a schedule - as a job it would starve the work it
    /// starts. A timer pauses with the queue: it does not tick while paused and
    /// ticks once immediately after resume.
    pub fn timer<F, Fut, E>(&self, interval: Duration, tick: F)
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: std::fmt::Debug + Send + 'static,
    {
        let book = Arc::clone(&self.0);
        tokio::spawn(async move {
            loop {
                wait_until_resumed(&book).await;
                if let Err(error) = tick().await {
                    eprintln!("timer failed: {error:?}");
                }
                tokio::select! {
                    _ = wait_until_paused(&book) => {}
                    _ = tokio::time::sleep(interval) => {}
                }
            }
        });
    }

    /// Stop starting new work, and ask running `Normal` jobs to park.
    ///
    /// `RealTime` work ignores the pause. A parked job keeps its place and its
    /// progress; resume runs it again from the factory.
    pub fn pause(&self) {
        if self.0.paused.swap(true, Ordering::SeqCst) {
            return;
        }
        let board = {
            let mut state = self.0.lock();
            let book = &mut *state;
            for pending in &book.queue {
                if !pending.priority.is_pausable() {
                    continue;
                }
                if let Some(job) = book
                    .board
                    .iter_mut()
                    .find(|job| job.key == pending.key && job.id == pending.id)
                {
                    job.state = JobState::Paused;
                }
            }
            for live in book.live.values() {
                if live.priority.is_pausable() {
                    live.run_cancel.cancel();
                }
            }
            book.snapshot()
        };
        self.0.publish(board);
    }

    /// Start dispatching again, including what parked itself during the pause.
    pub fn resume(&self) {
        if !self.0.paused.swap(false, Ordering::SeqCst) {
            return;
        }
        let board = {
            let mut state = self.0.lock();
            let book = &mut *state;
            for pending in &book.queue {
                if let Some(job) = book
                    .board
                    .iter_mut()
                    .find(|job| job.key == pending.key && job.id == pending.id)
                    && job.state == JobState::Paused
                {
                    job.state = JobState::Queued;
                }
            }
            book.snapshot()
        };
        self.0.publish(board);
        self.0.pump();
    }

    /// Whether the queue is currently refusing normal work.
    pub fn is_paused(&self) -> bool {
        self.0.is_paused()
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
    paused: AtomicBool,
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

    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    fn job_state(&self, priority: JobPriority) -> JobState {
        if self.is_paused() && priority.is_pausable() {
            JobState::Paused
        } else {
            JobState::Queued
        }
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
                let index = if self.is_paused() {
                    state
                        .queue
                        .iter()
                        .position(|pending| !pending.priority.is_pausable())
                } else if state.queue.is_empty() {
                    None
                } else {
                    Some(0)
                };
                let Some(index) = index else {
                    return;
                };
                let pending = state.queue.remove(index);
                let now = now_ms();
                if let Some(job) = state
                    .board
                    .iter_mut()
                    .find(|job| job.key == pending.key && job.id == pending.id)
                {
                    job.state = JobState::Running;
                    job.started_at = Some(now);
                }
                let run_cancel = CancellationToken::new();
                state.live.insert(
                    (pending.key, pending.id.clone()),
                    Live {
                        user_cancel: pending.user_cancel.clone(),
                        run_cancel: run_cancel.clone(),
                        priority: pending.priority,
                    },
                );
                (pending, run_cancel, state.snapshot())
            };
            let (pending, run_cancel, board) = started;
            self.publish(board);

            let book = Arc::clone(self);
            let Pending {
                key,
                id,
                priority,
                user_cancel,
                factory,
            } = pending;
            let ctx = Ctx {
                key,
                id: id.clone(),
                priority,
                user_cancel: user_cancel.clone(),
                cancel: run_cancel.clone(),
                book: Arc::clone(&book),
            };
            let body = (factory)(ctx);
            self.tracker.spawn(async move {
                let run = JobRun {
                    book: Arc::clone(&book),
                    key,
                    id: id.clone(),
                    priority,
                    user_cancel,
                    factory,
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
        priority: JobPriority,
        user_cancel: &CancellationToken,
        factory: &TaskFactory,
        outcome: Result<JobOutcome, String>,
    ) {
        if matches!(&outcome, Ok(JobOutcome::Paused)) && !user_cancel.is_cancelled() {
            self.park(key, id, priority, user_cancel, factory);
            return;
        }
        let state = if user_cancel.is_cancelled() {
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

    /// Put a paused job back in the queue instead of ending it.
    fn park(
        self: &Arc<Self>,
        key: JobKind,
        id: &JobId,
        priority: JobPriority,
        user_cancel: &CancellationToken,
        factory: &TaskFactory,
    ) {
        let board = {
            let mut state = self.lock();
            state.live.remove(&(key, id.clone()));
            if let Some(job) = state
                .board
                .iter_mut()
                .find(|job| job.key == key && job.id == *id)
            {
                job.state = JobState::Paused;
                job.started_at = None;
                job.finished_at = None;
            }
            state.queue.push(Pending {
                key,
                id: id.clone(),
                priority,
                user_cancel: user_cancel.clone(),
                factory: Arc::clone(factory),
            });
            state.queue.sort_by_key(|pending| pending.priority);
            state.snapshot()
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
            for ((key, id), live) in &book.live {
                if hit(*key, id) {
                    live.user_cancel.cancel();
                    live.run_cancel.cancel();
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
    /// Work waiting for a slot, highest priority first.
    queue: Vec<Pending>,
    /// Work running now, with both tokens that stop it.
    live: HashMap<LiveKey, Live>,
}

impl State {
    fn snapshot(&self) -> Vec<Job> {
        self.board.clone()
    }
}

struct Live {
    /// Set only by a user cancel, so pause and cancel stay distinguishable.
    user_cancel: CancellationToken,
    /// Set by either a user cancel or the queue's pause.
    run_cancel: CancellationToken,
    priority: JobPriority,
}

struct Pending {
    key: JobKind,
    id: JobId,
    priority: JobPriority,
    user_cancel: CancellationToken,
    factory: TaskFactory,
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
    priority: JobPriority,
    user_cancel: CancellationToken,
    factory: TaskFactory,
    reported: bool,
}

impl JobRun {
    fn report(mut self, outcome: Result<JobOutcome, String>) {
        self.reported = true;
        self.book.complete(
            self.key,
            &self.id,
            self.priority,
            &self.user_cancel,
            &self.factory,
            outcome,
        );
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
            self.priority,
            &self.user_cancel,
            &self.factory,
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
    priority: JobPriority,
    user_cancel: CancellationToken,
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

    /// The token of this run, for whatever needs to race a stop.
    ///
    /// It fires both when the user cancels and when the queue pauses, so a task
    /// can wait on it and then ask [`Self::pause_token`] why it stopped.
    pub fn cancel(&self) -> &CancellationToken {
        &self.cancel
    }

    /// Whether the user cancelled this item, as opposed to the queue pausing it.
    pub fn cancelled_by_user(&self) -> bool {
        self.user_cancel.is_cancelled()
    }

    /// This job's view of the queue's pause flag.
    pub fn pause_token(&self) -> PauseToken {
        PauseToken {
            book: Arc::clone(&self.book),
            pausable: self.priority.is_pausable(),
        }
    }

    /// Run one stop-aware segment of a task.
    ///
    /// The segment is cancelled when the user cancels or when a pausable job
    /// sees the queue pause. The returned [`Ran`] also answers whether the pause
    /// is what stopped it, which is what turns an early return into
    /// [`JobOutcome::Paused`] instead of a failure.
    pub async fn run_paused<T, F, Fut>(&self, work: F) -> Ran<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let work = work();
        tokio::pin!(work);
        let paused = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&paused);
        let run = self.cancel.clone();
        let user = self.user_cancel.clone();
        let pause = self.pause_token();
        let watch = tokio::spawn(async move {
            tokio::select! {
                _ = user.cancelled() => {}
                _ = pause.wait() => {
                    flag.store(true, Ordering::SeqCst);
                    run.cancel();
                }
            }
        });
        let value = work.await;
        watch.abort();
        // The watcher only wakes on its own poll, and the queue cancels the
        // token at the same moment it sets the pause flag, so the work can stop
        // before the watcher ran. Asking the flag directly covers that window.
        Ran {
            value,
            paused: paused.load(Ordering::SeqCst) || self.pause_token().requested(),
        }
    }

    /// Replace the done count, for work that resumes at a known offset.
    pub fn set_done(&self, done: u64) {
        self.modify(|progress| progress.done = done);
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

/// A job's view of the queue's pause flag.
#[derive(Clone)]
pub struct PauseToken {
    book: Arc<Book>,
    pausable: bool,
}

impl PauseToken {
    /// Whether the queue is paused and this job should stop at a safe point.
    pub fn requested(&self) -> bool {
        self.pausable && self.book.is_paused()
    }

    /// Resolve when the queue pauses. `RealTime` never resolves.
    pub async fn wait(&self) {
        if !self.pausable {
            std::future::pending::<()>().await;
            return;
        }
        while !self.book.is_paused() {
            tokio::time::sleep(PAUSE_POLL).await;
        }
    }
}

async fn wait_until_paused(book: &Book) {
    while !book.is_paused() {
        tokio::time::sleep(PAUSE_POLL).await;
    }
}

async fn wait_until_resumed(book: &Book) {
    while book.is_paused() {
        tokio::time::sleep(PAUSE_POLL).await;
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as i64)
}
