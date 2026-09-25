use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

use tokio::sync::Notify;

use crate::types::Pending;

#[derive(Default)]
pub struct RunQueue {
    pending: Mutex<VecDeque<Pending>>,
    waiting: Notify,
    dropped: AtomicUsize,
}

impl RunQueue {
    pub const CAPACITY: usize = 100;

    pub fn push(&self, run: Pending) -> Option<Pending> {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let dropped = if pending.len() >= Self::CAPACITY {
            let count = self.dropped.fetch_add(1, Ordering::SeqCst) + 1;
            tracing::warn!(
                dropped = count,
                "the automation queue is full; the oldest run was dropped"
            );
            pending.pop_front()
        } else {
            None
        };
        pending.push_back(run);
        self.waiting.notify_one();
        dropped
    }

    pub fn take(&self) -> Option<Pending> {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
    }

    pub async fn next(&self) -> Pending {
        loop {
            if let Some(run) = self.take() {
                return run;
            }
            self.waiting.notified().await;
        }
    }

    pub fn len(&self) -> usize {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
