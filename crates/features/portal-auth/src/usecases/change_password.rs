use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::helpers::credential_of;
use crate::repositories::{origin, position, set_hash};
use crate::services::{
    CHANGE_USERS, SessionStore, checked_password, hashed, may_touch, require_editable, users_of,
    users_view,
};
use crate::types::{Caller, UsersView};

pub const UNKNOWN: &str = "no such user";

#[derive(Clone)]
pub struct ChangePassword {
    configuration: Arc<ConfigStore>,
    sessions: Arc<SessionStore>,
}

impl ChangePassword {
    pub fn new(configuration: Arc<ConfigStore>, sessions: Arc<SessionStore>) -> ChangePassword {
        ChangePassword {
            configuration,
            sessions,
        }
    }

    pub async fn run(
        &self,
        caller: &Caller,
        name: &str,
        password: String,
        revision: &Revision,
    ) -> Result<Revisioned<UsersView>, ApiError> {
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let actor = &caller.principal;
        let own = actor.name == name;
        if !own && !actor.rights.allows(CHANGE_USERS) {
            return Err(ApiError::Forbidden(format!("needs {CHANGE_USERS}")));
        }
        checked_password(&password).map_err(|error| ApiError::Invalid(vec![error]))?;
        let target = origin(&snapshot, name).ok_or(ApiError::NotFound(UNKNOWN))?;
        if !own {
            may_touch(actor, &users_of(&snapshot.document)?, name)?;
        }
        let password_hash = hashed(password).await?;
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                let index = position(document, name).ok_or(ApiError::NotFound(UNKNOWN))?;
                set_hash(document, index, &password_hash);
                Ok(())
            })
            .await?;
        if own && let Some(token) = &caller.token {
            self.sessions.restamp(token, &credential_of(&password_hash));
        }
        Ok(Revisioned::new(
            users_view(&written.document, &actor.name)?,
            written.revision,
        ))
    }
}
