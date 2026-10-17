//! Stopping work in progress from another thread: the window's Cancel
//! button.

use crate::job::Job;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// A handle that stops what it is given to: a download of a tool, or yt-dlp
/// with every program it started. Clones share the same state.
#[derive(Clone, Default)]
pub struct Stop(Arc<Inner>);

#[derive(Default)]
struct Inner {
    stopped: AtomicBool,
    /// The programs running now, ended at once by [`Stop::stop`].
    running: Mutex<Option<Arc<Job>>>,
}

impl Stop {
    pub fn new() -> Stop {
        Stop::default()
    }

    /// Stops the work now and anything started with this handle later.
    pub fn stop(&self) {
        self.0.stopped.store(true, Ordering::SeqCst);
        if let Some(job) = self.lock().take() {
            job.kill();
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.0.stopped.load(Ordering::SeqCst)
    }

    /// `Err(Stopped)` once stopped, for `?`.
    pub fn check(&self) -> Result<(), Stopped> {
        if self.is_stopped() {
            Err(Stopped)
        } else {
            Ok(())
        }
    }

    /// Ends `job` when this handle stops; at once if it already has.
    pub(crate) fn watch(&self, job: Arc<Job>) {
        let mut running = self.lock();
        if self.is_stopped() {
            job.kill();
        } else {
            *running = Some(job);
        }
    }

    pub(crate) fn unwatch(&self) {
        self.lock().take();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Arc<Job>>> {
        // The lock only guards a handle; a panic elsewhere cannot leave it
        // half changed.
        self.0
            .running
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// The error of work that was stopped on purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopped;

impl fmt::Display for Stopped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stopped.")
    }
}

impl std::error::Error for Stopped {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_stop_together() {
        let stop = Stop::new();
        let other = stop.clone();
        assert_eq!(other.check(), Ok(()));
        stop.stop();
        assert!(other.is_stopped());
        assert_eq!(other.check(), Err(Stopped));
    }
}
