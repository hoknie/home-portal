use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use time::OffsetDateTime;

use crate::types::Reception;

#[derive(Default)]
pub struct WebhookBook {
    receptions: Mutex<HashMap<String, Reception>>,
    calls: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl WebhookBook {
    pub const CALLS_PER_WINDOW: usize = 60;
    pub const WINDOW: Duration = Duration::from_secs(60);

    pub fn allow(&self, id: &str, now: Instant) -> Result<(), u64> {
        let mut calls = self.calls.lock().unwrap_or_else(PoisonError::into_inner);
        let recent = calls.entry(id.to_string()).or_default();
        while recent
            .front()
            .is_some_and(|call| now.saturating_duration_since(*call) >= Self::WINDOW)
        {
            recent.pop_front();
        }
        if let Some(oldest) = recent.front()
            && recent.len() >= Self::CALLS_PER_WINDOW
        {
            let wait = Self::WINDOW.saturating_sub(now.saturating_duration_since(*oldest));
            return Err(wait.as_secs().max(1));
        }
        recent.push_back(now);
        Ok(())
    }

    pub fn record(&self, id: &str, status: u16) {
        self.receptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(
                id.to_string(),
                Reception {
                    at: OffsetDateTime::now_utc(),
                    status,
                },
            );
    }

    pub fn last(&self, id: &str) -> Option<Reception> {
        self.receptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .copied()
    }
}
