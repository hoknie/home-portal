use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};

use time::{Duration, OffsetDateTime};

use crate::types::Attempts;

#[derive(Default)]
pub struct Throttle {
    attempts: Mutex<HashMap<IpAddr, Attempts>>,
}

impl Throttle {
    pub const FAILURES_BEFORE_LOCK: u32 = 5;
    pub const WINDOW: Duration = Duration::minutes(15);
    pub const LOCK: Duration = Duration::seconds(60);
    pub const CAPACITY: usize = 10_000;

    pub fn check(&self, client: IpAddr, now: OffsetDateTime) -> Result<(), u64> {
        let mut attempts = self.attempts.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(entry) = attempts.get(&client) else {
            return Ok(());
        };
        match entry.locked_until {
            Some(until) if until > now => Err((until - now).whole_seconds().max(1).unsigned_abs()),
            Some(_) => {
                attempts.remove(&client);
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn fail(&self, client: IpAddr, now: OffsetDateTime) {
        let mut attempts = self.attempts.lock().unwrap_or_else(PoisonError::into_inner);
        if !attempts.contains_key(&client)
            && attempts.len() >= Self::CAPACITY
            && let Some(oldest) = attempts
                .iter()
                .min_by_key(|(_, entry)| entry.first_failure_at)
                .map(|(address, _)| *address)
        {
            attempts.remove(&oldest);
        }
        let entry = attempts.entry(client).or_insert(Attempts {
            failures: 0,
            first_failure_at: now,
            locked_until: None,
        });
        if now - entry.first_failure_at > Self::WINDOW {
            entry.failures = 0;
            entry.first_failure_at = now;
        }
        entry.failures += 1;
        if entry.failures >= Self::FAILURES_BEFORE_LOCK {
            entry.locked_until = Some(now + Self::LOCK);
        }
    }

    pub fn succeed(&self, client: IpAddr) {
        self.attempts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&client);
    }

    #[cfg(test)]
    pub fn tracked(&self) -> usize {
        self.attempts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}
