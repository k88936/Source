//! Running a batch of independent attempts: concurrency, backoff, and stopping.
//!
//! The runner knows nothing about albums, files or the server. A use case hands
//! it the keys to visit and one closure that performs a single attempt; what a
//! visit means - rows, progress, events - stays in the use case.

use std::future::Future;
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
        let in_flight = keys.len().max(1);
        let semaphore = Semaphore::new(self.concurrency);
        let retry = self.retry;

        stream::iter(keys)
            .map(|key| run_one(&semaphore, retry, cancel, &attempt, key))
            .buffer_unordered(in_flight)
            .collect::<Vec<()>>()
            .await;
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
