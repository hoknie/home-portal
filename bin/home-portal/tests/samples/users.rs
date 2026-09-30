use std::collections::BTreeMap;

use portal_auth::{AreaResponse, GroupResponse, GroupsResponse, UserResponse, UsersResponse};

use crate::check;

#[test]
fn the_users_sample_matches_its_serializer() {
    let users = UsersResponse {
        users: vec![
            UserResponse {
                name: "admin".into(),
                group: Some("admin".into()),
                you: true,
            },
            UserResponse {
                name: "anna".into(),
                group: Some("family".into()),
                you: false,
            },
            UserResponse {
                name: "guest".into(),
                group: None,
                you: false,
            },
        ],
        editable: true,
    };
    check("users", serde_json::to_value(users).unwrap());
}

#[test]
fn the_groups_sample_matches_its_serializer() {
    let rights = |pairs: &[(&str, &[&str])]| -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(area, actions)| {
                (
                    area.to_string(),
                    actions.iter().map(|action| action.to_string()).collect(),
                )
            })
            .collect()
    };
    let groups = GroupsResponse {
        groups: vec![
            GroupResponse {
                name: "admin".into(),
                builtin: true,
                rights: AreaResponse::matrix()
                    .into_iter()
                    .map(|area| {
                        (
                            area.area.to_string(),
                            area.actions.into_iter().map(str::to_string).collect(),
                        )
                    })
                    .collect(),
                members: vec!["admin".into()],
            },
            GroupResponse {
                name: "family".into(),
                builtin: false,
                rights: rights(&[
                    ("automations", &["read", "execute"]),
                    ("services", &["update"]),
                ]),
                members: vec!["anna".into()],
            },
            GroupResponse {
                name: "guests".into(),
                builtin: false,
                rights: BTreeMap::new(),
                members: Vec::new(),
            },
        ],
        matrix: AreaResponse::matrix(),
    };
    check("groups", serde_json::to_value(groups).unwrap());
}
