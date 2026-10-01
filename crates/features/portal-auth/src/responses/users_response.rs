use schemars::JsonSchema;
use serde::Serialize;

use super::UserResponse;
use crate::types::UsersView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UsersResponse {
    pub users: Vec<UserResponse>,
    pub editable: bool,
}

impl UsersResponse {
    pub fn of(view: UsersView) -> UsersResponse {
        UsersResponse {
            users: view
                .members
                .into_iter()
                .map(|(name, group)| UserResponse {
                    you: name == view.you,
                    name,
                    group,
                })
                .collect(),
            editable: view.editable,
        }
    }
}
