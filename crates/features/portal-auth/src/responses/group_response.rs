use std::collections::BTreeMap;

use serde::Serialize;

use super::session_response::rights_map;
use crate::types::GroupView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroupResponse {
    pub name: String,
    pub builtin: bool,
    pub rights: BTreeMap<String, Vec<String>>,
    pub members: Vec<String>,
}

impl GroupResponse {
    pub fn of(view: &GroupView) -> GroupResponse {
        GroupResponse {
            name: view.name.clone(),
            builtin: view.builtin,
            rights: rights_map(&view.rights),
            members: view.members.clone(),
        }
    }
}
