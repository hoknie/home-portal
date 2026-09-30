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

    pub fn reserve(&self, client: IpAddr, now: OffsetDateTime) -> Result<(), u64> {
        let mut attempts = self.attempts.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(entry) = attempts.get(&client) {
            match entry.locked_until {
                Some(until) if until > now => {
                    return Err((until - now).whole_seconds().max(1).unsigned_abs());
                }
                Some(_) => {
                    attempts.remove(&client);
                }
                None => {}
            }
        }
        Self::make_room(&mut attempts, client);
        let entry = attempts.entry(client).or_insert(Attempts {
            failures: 0,
            first_failure_at: now,
            locked_until: None,
            in_flight: 0,
        });
        if entry.in_flight == 0 && now - entry.first_failure_at > Self::WINDOW {
            entry.failures = 0;
            entry.first_failure_at = now;
        }
        if entry.failures + entry.in_flight >= Self::FAILURES_BEFORE_LOCK {
            return Err(Self::LOCK.whole_seconds().unsigned_abs());
        }
        entry.in_flight += 1;
        Ok(())
    }

    pub fn release(&self, client: IpAddr) {
        let mut attempts = self.attempts.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(entry) = attempts.get_mut(&client) {
            entry.in_flight = entry.in_flight.saturating_sub(1);
            if entry.in_flight == 0 && entry.failures == 0 {
                attempts.remove(&client);
            }
        }
    }

    fn make_room(attempts: &mut HashMap<IpAddr, Attempts>, client: IpAddr) {
        if !attempts.contains_key(&client)
            && attempts.len() >= Self::CAPACITY
            && let Some(oldest) = attempts
                .iter()
                .min_by_key(|(_, entry)| entry.first_failure_at)
                .map(|(address, _)| *address)
        {
            attempts.remove(&oldest);
        }
    }

    pub fn fail(&self, client: IpAddr, now: OffsetDateTime) {
        let mut attempts = self.attempts.lock().unwrap_or_else(PoisonError::into_inner);
        Self::make_room(&mut attempts, client);
        let entry = attempts.entry(client).or_insert(Attempts {
            failures: 0,
            first_failure_at: now,
            locked_until: None,
            in_flight: 0,
        });
        entry.in_flight = entry.in_flight.saturating_sub(1);
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
