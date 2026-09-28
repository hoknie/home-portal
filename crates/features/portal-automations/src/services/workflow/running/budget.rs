use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::Instant;

use crate::types::{Ending, Workflow};

pub struct Budget {
    deadline: Instant,
    executed: AtomicUsize,
    stop: watch::Receiver<bool>,
}

impl Budget {
    pub fn new(timeout: Duration, stop: watch::Receiver<bool>) -> Budget {
        Budget {
            deadline: Instant::now() + timeout,
            executed: AtomicUsize::new(0),
            stop,
        }
    }

    pub fn admit(&self) -> Result<(), Ending> {
        if *self.stop.borrow() {
            return Err(Ending::Stopped);
        }
        if Instant::now() >= self.deadline {
            return Err(Ending::TimedOut);
        }
        let executed = self.executed.fetch_add(1, Ordering::SeqCst) + 1;
        if executed > Workflow::MOST_STEPS_RUN {
            return Err(Ending::Failed(format!(
                "ran more than {} steps",
                Workflow::MOST_STEPS_RUN
            )));
        }
        Ok(())
    }

    pub fn stop_signal(&self) -> watch::Receiver<bool> {
        self.stop.clone()
    }

    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    pub async fn guard<F: Future>(&self, work: F) -> Result<F::Output, Ending> {
        let mut stop = self.stop.clone();
        tokio::select! {
            done = work => Ok(done),
            _ = tokio::time::sleep_until(self.deadline) => Err(Ending::TimedOut),
            _ = stop.wait_for(|stopped| *stopped) => Err(Ending::Stopped),
        }
    }

    pub async fn pause(&self, length: Duration) -> Result<(), Ending> {
        self.guard(tokio::time::sleep(length)).await
    }
}
