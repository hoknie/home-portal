use portal_feature::{ApiError, Module, ModuleSwitches};
use toml_edit::DocumentMut;

use crate::types::{UsersSection, UsersView};

pub const MODULE_OFF: &str = "the users module is off; switch it on under Management → Modules";

pub fn users_editable(document: &DocumentMut) -> bool {
    ModuleSwitches::resolve(document)
        .unwrap_or_default()
        .is_on(Module::Users)
}

pub fn require_editable(document: &DocumentMut) -> Result<(), ApiError> {
    if users_editable(document) {
        Ok(())
    } else {
        Err(ApiError::Conflict(MODULE_OFF.to_string()))
    }
}

pub fn users_of(document: &DocumentMut) -> Result<UsersSection, ApiError> {
    UsersSection::read(document)
        .map_err(|message| ApiError::Internal(format!("users section: {message}")))
}

pub fn users_view(document: &DocumentMut, you: &str) -> Result<UsersView, ApiError> {
    Ok(UsersView {
        names: users_of(document)?
            .users
            .into_iter()
            .map(|user| user.name)
            .collect(),
        you: you.to_string(),
        editable: users_editable(document),
    })
}

pub async fn hashed(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || crate::helpers::hash_password(&password))
        .await
        .map_err(|error| ApiError::Internal(format!("password hashing: {error}")))?
        .map_err(|message| ApiError::Internal(format!("password hashing: {message}")))
}
