use std::collections::BTreeMap;

use portal_feature::{Principal, Rights};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionResponse {
    pub name: String,
    pub group: Option<String>,
    pub admin: bool,
    pub rights: BTreeMap<String, Vec<String>>,
}

impl SessionResponse {
    pub fn of(principal: &Principal) -> SessionResponse {
        SessionResponse {
            name: principal.name.clone(),
            group: principal.group.clone(),
            admin: principal.rights.is_admin(),
            rights: rights_map(&principal.rights),
        }
    }
}

pub fn rights_map(rights: &Rights) -> BTreeMap<String, Vec<String>> {
    rights
        .by_area()
        .into_iter()
        .map(|(area, actions)| {
            (
                area.name().to_string(),
                actions
                    .into_iter()
                    .map(|action| action.name().to_string())
                    .collect(),
            )
        })
        .collect()
}
