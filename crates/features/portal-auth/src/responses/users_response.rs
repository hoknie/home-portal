use serde::Serialize;

use super::UserResponse;
use crate::types::UsersView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsersResponse {
    pub users: Vec<UserResponse>,
    pub editable: bool,
}

impl UsersResponse {
    pub fn of(view: UsersView) -> UsersResponse {
        UsersResponse {
            users: view
                .names
                .into_iter()
                .map(|name| UserResponse {
                    you: name == view.you,
                    name,
                })
                .collect(),
            editable: view.editable,
        }
    }
}
