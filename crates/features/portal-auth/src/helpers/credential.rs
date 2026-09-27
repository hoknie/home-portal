use super::token_hash;

pub const CREDENTIAL_LENGTH: usize = 16;

pub fn credential_of(password_hash: &str) -> String {
    token_hash(password_hash)[..CREDENTIAL_LENGTH].to_string()
}
