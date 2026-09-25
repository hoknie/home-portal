use sha2::{Digest, Sha256};

pub fn digest_of(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
