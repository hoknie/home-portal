use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

use tokio::sync::Notify;

use crate::types::Outgoing;

#[derive(Default)]
pub struct Outbox {
    messages: Mutex<VecDeque<Outgoing>>,
    waiting: Notify,
    dropped: AtomicUsize,
}

impl Outbox {
    pub const CAPACITY: usize = 100;

    pub fn push(&self, message: Outgoing) {
        let mut messages = self.messages.lock().unwrap_or_else(PoisonError::into_inner);
        if messages.len() >= Self::CAPACITY {
            messages.pop_front();
            let dropped = self.dropped.fetch_add(1, Ordering::SeqCst) + 1;
            tracing::warn!(
                dropped,
                "the telegram queue is full; the oldest message was dropped"
            );
        }
        messages.push_back(message);
        self.waiting.notify_one();
    }

    pub fn take(&self) -> Option<Outgoing> {
        self.messages
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
    }

    pub async fn next(&self) -> Outgoing {
        loop {
            if let Some(message) = self.take() {
                return message;
            }
            self.waiting.notified().await;
        }
    }

    pub fn waiting(&self) -> usize {
        self.messages
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    pub fn dropped(&self) -> usize {
        self.dropped.load(Ordering::SeqCst)
    }
}
