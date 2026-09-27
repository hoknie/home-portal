use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use time::{Duration, OffsetDateTime};

use crate::helpers::{new_token, token_hash};
use crate::repositories::SessionFile;
use crate::types::Session;

#[derive(Default)]
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Session>>,
    file: Option<SessionFile>,
    extended: AtomicBool,
}

impl SessionStore {
    pub const LIFETIME: Duration = Duration::days(7);

    pub fn kept_in(file: SessionFile, now: OffsetDateTime) -> SessionStore {
        let mut sessions = file.load();
        sessions.retain(|_, session| session.expires_at > now);
        SessionStore {
            sessions: Mutex::new(sessions),
            file: Some(file),
            extended: AtomicBool::new(false),
        }
    }

    pub fn create(&self, name: &str, credential: &str, now: OffsetDateTime) -> String {
        let token = new_token();
        let session = Session {
            name: name.to_string(),
            expires_at: now + Self::LIFETIME,
            credential: Some(credential.to_string()),
        };
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        sessions.insert(token_hash(&token), session);
        self.save(&sessions);
        token
    }

    pub fn admit(
        &self,
        token: &str,
        now: OffsetDateTime,
        credential_of: impl Fn(&str) -> Option<String>,
    ) -> Option<String> {
        let key = token_hash(token);
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        let session = sessions.get_mut(&key)?;
        let current = credential_of(&session.name);
        let valid = session.expires_at > now
            && match (&current, &session.credential) {
                (None, _) => false,
                (Some(current), Some(remembered)) => current == remembered,
                (Some(_), None) => true,
            };
        if !valid {
            sessions.remove(&key);
            self.save(&sessions);
            return None;
        }
        if session.credential.is_none() {
            session.credential = current;
        }
        session.expires_at = now + Self::LIFETIME;
        self.extended.store(true, Ordering::Relaxed);
        Some(session.name.clone())
    }

    pub fn restamp(&self, token: &str, credential: &str) {
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(session) = sessions.get_mut(&token_hash(token)) {
            session.credential = Some(credential.to_string());
            self.save(&sessions);
        }
    }

    pub fn end(&self, token: &str) {
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        if sessions.remove(&token_hash(token)).is_some() {
            self.save(&sessions);
        }
    }

    pub fn prune(&self, now: OffsetDateTime) -> usize {
        let mut sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
        let before = sessions.len();
        sessions.retain(|_, session| session.expires_at > now);
        let pruned = before - sessions.len();
        if pruned > 0 || self.extended.load(Ordering::Relaxed) {
            self.save(&sessions);
        }
        pruned
    }

    pub fn flush(&self) {
        if self.extended.load(Ordering::Relaxed) {
            let sessions = self.sessions.lock().unwrap_or_else(PoisonError::into_inner);
            self.save(&sessions);
        }
    }

    fn save(&self, sessions: &HashMap<String, Session>) {
        let Some(file) = &self.file else {
            return;
        };
        self.extended.store(false, Ordering::Relaxed);
        if let Err(error) = file.save(sessions) {
            tracing::warn!(path = %file.path().display(), %error, "cannot write the sessions file; sessions will not survive a restart");
        }
    }
}
