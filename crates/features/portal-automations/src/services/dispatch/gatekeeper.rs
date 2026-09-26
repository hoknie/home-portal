use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use crate::types::{Admission, Automation, SkipReason};

#[derive(Default)]
pub struct Gatekeeper {
    states: Mutex<HashMap<String, Admission>>,
}

impl Gatekeeper {
    pub const STARTS_PER_WINDOW: usize = 60;
    pub const WINDOW: Duration = Duration::from_secs(3600);

    pub fn admit(&self, automation: &Automation, now: Instant) -> Result<(), SkipReason> {
        let mut states = self.states.lock().unwrap_or_else(PoisonError::into_inner);
        let state = states.entry(automation.id.clone()).or_default();
        let cooldown = Duration::from_secs(automation.cooldown_seconds);
        if state
            .last_start
            .is_some_and(|last| now.saturating_duration_since(last) < cooldown)
        {
            return Err(SkipReason::Cooldown);
        }
        if state.running {
            return Err(SkipReason::Running);
        }
        if state.pending {
            return Err(SkipReason::Pending);
        }
        while state
            .starts
            .front()
            .is_some_and(|start| now.saturating_duration_since(*start) >= Self::WINDOW)
        {
            state.starts.pop_front();
        }
        if state.starts.len() >= Self::STARTS_PER_WINDOW {
            return Err(SkipReason::RateLimit);
        }
        state.pending = true;
        state.last_start = Some(now);
        Ok(())
    }

    pub fn dequeued(&self, automation: &str) {
        self.update(automation, |state| {
            state.pending = false;
            state.last_start = None;
        });
    }

    pub fn started(&self, automation: &str, now: Instant) {
        self.update(automation, |state| {
            state.pending = false;
            state.running = true;
            state.starts.push_back(now);
        });
    }

    pub fn finished(&self, automation: &str) {
        self.update(automation, |state| state.running = false);
    }

    pub fn busy(&self) -> bool {
        self.states
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .any(|state| state.running || state.pending)
    }

    #[cfg(test)]
    pub fn busy_with(&self, automation: &str) -> bool {
        self.states
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(automation)
            .is_some_and(|state| state.running || state.pending)
    }

    fn update(&self, automation: &str, change: impl FnOnce(&mut Admission)) {
        let mut states = self.states.lock().unwrap_or_else(PoisonError::into_inner);
        change(states.entry(automation.to_string()).or_default());
    }
}
