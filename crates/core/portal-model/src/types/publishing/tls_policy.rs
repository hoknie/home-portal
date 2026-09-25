use serde::{Deserialize, Serialize};

use super::TlsMode;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(default)]
pub struct TlsPolicy {
    pub mode: TlsMode,
    pub email: Option<String>,
    pub certificate: Option<String>,
    pub key: Option<String>,
}

impl TlsPolicy {
    pub const NEEDS_FILE: &'static str = "is required when mode is files";
    pub const INVALID_EMAIL: &'static str = "must be an email address";

    pub fn problems(&self) -> Vec<(&'static str, &'static str)> {
        let mut problems = Vec::new();
        if let Some(email) = &self.email
            && !Self::looks_like_email(email)
        {
            problems.push(("email", Self::INVALID_EMAIL));
        }
        if self.mode == TlsMode::Files {
            if self
                .certificate
                .as_deref()
                .is_none_or(|path| path.trim().is_empty())
            {
                problems.push(("certificate", Self::NEEDS_FILE));
            }
            if self
                .key
                .as_deref()
                .is_none_or(|path| path.trim().is_empty())
            {
                problems.push(("key", Self::NEEDS_FILE));
            }
        }
        problems
    }

    fn looks_like_email(email: &str) -> bool {
        email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !email.contains(char::is_whitespace)
        })
    }
}
