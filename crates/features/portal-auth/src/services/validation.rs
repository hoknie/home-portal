use std::collections::HashSet;

use portal_feature::FieldError;
use toml_edit::DocumentMut;

use crate::helpers::is_argon2id;
use crate::types::UsersSection;

pub const NO_USERS: &str = "at least one user is required; run `home-portal password-hash` and add a [[users]] entry with name and password_hash";

pub fn validate_users(document: &DocumentMut) -> Vec<FieldError> {
    let section = match UsersSection::read(document) {
        Ok(section) => section,
        Err(message) => return vec![FieldError::new("users", message)],
    };
    if section.users.is_empty() {
        return vec![FieldError::new("users", NO_USERS)];
    }
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (index, user) in section.users.iter().enumerate() {
        if user.name.trim().is_empty() {
            errors.push(FieldError::new(
                format!("users[{index}].name"),
                "must not be empty",
            ));
        }
        if !seen.insert(user.name.as_str()) {
            errors.push(FieldError::new(
                format!("users[{index}].name"),
                "is used by another user",
            ));
        }
        if !is_argon2id(&user.password_hash) {
            errors.push(FieldError::new(
                format!("users[{index}].password_hash"),
                "must be an argon2id hash printed by `home-portal password-hash`",
            ));
        }
    }
    errors
}
