use std::sync::OnceLock;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2};

pub const DUMMY_PASSWORD: &str = "a password nobody signs in with";

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| error.to_string())
}

pub fn is_argon2id(hash: &str) -> bool {
    PasswordHash::new(hash)
        .is_ok_and(|parsed| parsed.algorithm.as_str() == Algorithm::Argon2id.as_str())
}

pub fn verify_password(password: &str, hash: Option<&str>) -> bool {
    let candidate = hash.unwrap_or_else(|| dummy_hash());
    let verified = PasswordHash::new(candidate)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false);
    verified && hash.is_some()
}

fn dummy_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| hash_password(DUMMY_PASSWORD).unwrap_or_default())
}
