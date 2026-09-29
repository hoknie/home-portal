use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Section};
use portal_feature::ApiError;

use crate::repositories::{append, last_origin};
use crate::services::{
    checked_name, checked_password, hashed, require_editable, users_of, users_view,
};
use crate::types::UsersView;

#[derive(Clone)]
pub struct CreateUser {
    configuration: Arc<ConfigStore>,
}

impl CreateUser {
    pub fn new(configuration: Arc<ConfigStore>) -> CreateUser {
        CreateUser { configuration }
    }

    pub async fn run(
        &self,
        you: &str,
        name: &str,
        password: String,
        revision: &Revision,
    ) -> Result<Revisioned<UsersView>, ApiError> {
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let users = users_of(&snapshot.document)?;
        let mut errors = Vec::new();
        let name = checked_name(name, &users)
            .map_err(|error| errors.push(error))
            .ok();
        if let Err(error) = checked_password(&password) {
            errors.push(error);
        }
        let Some(name) = name.filter(|_| errors.is_empty()) else {
            return Err(ApiError::Invalid(errors));
        };
        let target =
            last_origin(&snapshot).unwrap_or_else(|| self.configuration.home_of(Section::Users));
        let password_hash = hashed(password).await?;
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                append(document, &name, &password_hash);
                Ok(())
            })
            .await?;
        Ok(Revisioned::new(
            users_view(&written.document, you)?,
            written.revision,
        ))
    }
}
