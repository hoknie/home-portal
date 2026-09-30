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
        let mut forms: Vec<String> = revealed
            .iter()
            .flat_map(|secret| forms_of(secret))
            .collect();
        forms.sort_by_key(|form| std::cmp::Reverse(form.len()));
        forms.dedup();
        let mut masked = text.to_string();
        for form in &forms {
            masked = masked.replace(form.as_str(), MASK);
        }
        for secret in revealed.iter() {
            masked = masked_edges(&masked, secret);
        }
        masked
    }
}

pub const SHORTEST_EDGE: usize = 4;

fn forms_of(secret: &str) -> Vec<String> {
    let escaped = serde_json::to_string(secret)
        .map(|quoted| quoted[1..quoted.len() - 1].to_string())
        .unwrap_or_default();
    let percent: String = secret
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect();
    [secret.to_string(), escaped, percent]
        .into_iter()
        .filter(|form| !form.is_empty())
        .collect()
}

fn masked_edges(text: &str, secret: &str) -> String {
    let mut masked = text.to_string();
    let longest = secret.len().saturating_sub(1).min(masked.len());
    if let Some(cut) = (SHORTEST_EDGE..=longest).rev().find(|length| {
        secret.is_char_boundary(secret.len() - length)
            && masked.is_char_boundary(*length)
            && masked.starts_with(&secret[secret.len() - length..])
    }) {
        masked = format!("{MASK}{}", &masked[cut..]);
    }
    let longest = secret.len().saturating_sub(1).min(masked.len());
    if let Some(cut) = (SHORTEST_EDGE..=longest).rev().find(|length| {
        secret.is_char_boundary(*length)
            && masked.is_char_boundary(masked.len() - length)
            && masked.ends_with(&secret[..*length])
    }) {
        masked = format!("{}{MASK}", &masked[..masked.len() - cut]);
    }
    masked
}
