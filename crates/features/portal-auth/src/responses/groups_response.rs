use schemars::JsonSchema;
use serde::Serialize;

use super::{AreaResponse, GroupResponse};
use crate::types::GroupsView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct GroupsResponse {
    pub groups: Vec<GroupResponse>,
    pub matrix: Vec<AreaResponse>,
}

impl GroupsResponse {
    pub fn of(view: &GroupsView) -> GroupsResponse {
        GroupsResponse {
            groups: view.groups.iter().map(GroupResponse::of).collect(),
            matrix: AreaResponse::matrix(),
        }
    }
}
