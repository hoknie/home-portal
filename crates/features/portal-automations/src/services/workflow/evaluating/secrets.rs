use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, PoisonError};

pub const MASK: &str = "***";

pub type SecretLookup = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

pub struct Secrets {
    lookup: SecretLookup,
    revealed: Mutex<BTreeSet<String>>,
}

impl Secrets {
    pub fn new(lookup: SecretLookup) -> Secrets {
        Secrets {
            lookup,
            revealed: Mutex::new(BTreeSet::new()),
        }
    }

    #[cfg(test)]
    pub fn none() -> Secrets {
        Secrets::new(Arc::new(|_| None))
    }

    pub fn reveal(&self, key: &str) -> Result<String, String> {
        let value = (self.lookup)(key).ok_or_else(|| format!("the secret {key} is not set"))?;
        if !value.is_empty() {
            self.revealed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(value.clone());
        }
        Ok(value)
    }

    pub fn mask(&self, text: &str) -> String {
        let revealed = self.revealed.lock().unwrap_or_else(PoisonError::into_inner);
        let mut masked = text.to_string();
        let mut longest_first: Vec<&String> = revealed.iter().collect();
        longest_first.sort_by_key(|secret| std::cmp::Reverse(secret.len()));
        for secret in longest_first {
            masked = masked.replace(secret.as_str(), MASK);
        }
        masked
    }
}
