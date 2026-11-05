//! Running a batch of independent attempts: concurrency, backoff, and stopping.
//!
//! The runner knows nothing about albums, files or the server. A use case hands
//! it the keys to visit and one closure that performs a single attempt; what a
//! visit means - rows, progress, events - stays in the use case.

use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::Duration;

use futures::StreamExt;
use futures::stream;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;

/// How many times one visit is attempted, and the wait before the first retry.
#[derive(Debug, Clone, Copy)]
pub struct Retry {
    /// Automatic retries after the first failure. `0` never retries.
    pub attempts: u32,
    /// The first wait; it doubles with every further retry.
    pub base: Duration,
}

impl Retry {
    pub const NONE: Retry = Retry {
        attempts: 0,
        base: Duration::ZERO,
    };

    /// Wait before retry number `retried` (1-based).
    pub fn delay(&self, retried: u32) -> Duration {
        let factor = 1u32 << retried.saturating_sub(1).min(4);
        self.base * factor
    }
}

impl Default for Retry {
    fn default() -> Self {
        Retry {
            attempts: 3,
            base: Duration::from_secs(3),
        }
    }
}

/// What an attempt is told about itself: its 0-based number, and the budget.
///
/// The budget is handed over so an attempt can tell the retry the runner will
/// make from the last one it will not make, and record a final failure itself.
#[derive(Debug, Clone, Copy)]
pub struct Try {
    pub number: u32,
    pub retry: Retry,
}

/// How many attempt futures are polled per concurrency slot.
const WINDOW_PER_SLOT: usize = 32;

/// A batch policy: how many attempts run at once, and how they are retried.
#[derive(Debug, Clone, Copy)]
pub struct Runner {
    concurrency: usize,
    retry: Retry,
}

impl Runner {
    pub fn new(concurrency: usize, retry: Retry) -> Self {
        Self {
            concurrency: concurrency.max(1),
            retry,
        }
    }

    /// Visit every key, at most [`Self::concurrency`] attempts at a time.
    ///
    /// An attempt holds its slot only while it runs: one sleeping in backoff
    /// gives the slot back, so a slow key never starves the rest. Stopping the
    /// token stops new attempts and cuts a backoff short; an attempt already
    /// running is left to finish.
    pub async fn run<T, F, Fut>(&self, keys: Vec<T>, cancel: &CancellationToken, attempt: F)
    where
        T: Clone + Send,
        F: Fn(T, Try) -> Fut + Sync,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        // The window is not the concurrency limit - a permit is - but it is
        // bounded, so a backlog of twenty thousand files never materialises
        // twenty thousand attempt futures at once.
        let window = keys.len().min(self.concurrency * WINDOW_PER_SLOT).max(1);
        let semaphore = Semaphore::new(self.concurrency);
        let retry = self.retry;

        stream::iter(keys)
            .map(|key| run_one(&semaphore, retry, cancel, &attempt, key))
            .buffer_unordered(window)
            .collect::<Vec<()>>()
            .await;
    }
}

/// The attempts running right now, so one of them can be stopped from outside.
///
/// A batch token stops a whole pass; this is the other direction - one key,
/// picked from a list, while the rest of the batch keeps going.
pub struct Attempts<K: Eq + Hash> {
    running: Mutex<HashMap<K, CancellationToken>>,
}

impl<K: Eq + Hash> Default for Attempts<K> {
    fn default() -> Self {
        Self {
            running: Mutex::new(HashMap::new()),
        }
    }
}

impl<K: Eq + Hash + Clone> Attempts<K> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an attempt before it touches anything, so a stop that lands
    /// during setup is not lost. The registration ends with the guard.
    pub fn open(&self, key: K) -> Attempt<'_, K> {
        let token = CancellationToken::new();
        self.running
            .lock()
            .unwrap()
            .insert(key.clone(), token.clone());
        Attempt {
            attempts: self,
            key,
            token,
        }
    }

    /// Stop the attempt running under `key`, if there is one.
    pub fn stop(&self, key: &K) {
        if let Some(token) = self.running.lock().unwrap().get(key) {
            token.cancel();
        }
    }
}

/// One registered attempt. The token it listens to lives as long as the guard.
pub struct Attempt<'a, K: Eq + Hash> {
    attempts: &'a Attempts<K>,
    key: K,
    token: CancellationToken,
}

impl<K: Eq + Hash> Attempt<'_, K> {
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }
}

impl<K: Eq + Hash> Drop for Attempt<'_, K> {
    fn drop(&mut self) {
        let mut running = self.attempts.running.lock().unwrap();
        // A newer attempt for the same key may have taken the slot already.
        if running.get(&self.key) == Some(&self.token) {
            running.remove(&self.key);
        }
    }
}

async fn run_one<T, F, Fut>(
    semaphore: &Semaphore,
    retry: Retry,
    cancel: &CancellationToken,
    attempt: &F,
    key: T,
) where
    T: Clone,
    F: Fn(T, Try) -> Fut,
    Fut: Future<Output = Result<(), AppError>>,
{
    let mut number = 0u32;
    loop {
        if cancel.is_cancelled() {
            return;
        }
        let result = {
            let Ok(_permit) = semaphore.acquire().await else {
                return;
            };
            attempt(key.clone(), Try { number, retry }).await
        };
        let Err(error) = result else {
            return;
        };
        if !error.retryable || number >= retry.attempts {
            return;
        }
        number += 1;
        // A stop during the wait is a stop, not one more attempt.
        tokio::select! {
            _ = cancel.cancelled() => return,
            _ = tokio::time::sleep(retry.delay(number)) => {}
        }
    }
}
