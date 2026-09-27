use portal_feature::FieldError;

use crate::types::UsersSection;

pub const NAME_FIELD: &str = "name";
pub const PASSWORD_FIELD: &str = "password";
pub const LONGEST_NAME: usize = 64;
pub const SHORTEST_PASSWORD: usize = 8;
pub const LONGEST_PASSWORD: usize = 1024;
pub const NAME_EMPTY: &str = "must not be empty";
pub const NAME_TOO_LONG: &str = "must be at most 64 characters";
pub const NAME_CONTROL: &str = "must not contain control characters";
pub const NAME_TAKEN: &str = "is used by another user";
pub const PASSWORD_TOO_SHORT: &str = "must be at least 8 characters";
pub const PASSWORD_TOO_LONG: &str = "must be at most 1024 characters";

pub fn checked_name(name: &str, users: &UsersSection) -> Result<String, FieldError> {
    let trimmed = name.trim();
    let problem = if trimmed.is_empty() {
        Some(NAME_EMPTY)
    } else if trimmed.chars().count() > LONGEST_NAME {
        Some(NAME_TOO_LONG)
    } else if trimmed.chars().any(char::is_control) {
        Some(NAME_CONTROL)
    } else if users.find(trimmed).is_some() {
        Some(NAME_TAKEN)
    } else {
        None
    };
    match problem {
        Some(message) => Err(FieldError::new(NAME_FIELD, message)),
        None => Ok(trimmed.to_string()),
    }
}

pub fn checked_password(password: &str) -> Result<(), FieldError> {
    let length = password.chars().count();
    if length < SHORTEST_PASSWORD {
        return Err(FieldError::new(PASSWORD_FIELD, PASSWORD_TOO_SHORT));
    }
    if length > LONGEST_PASSWORD {
        return Err(FieldError::new(PASSWORD_FIELD, PASSWORD_TOO_LONG));
    }
    Ok(())
}
