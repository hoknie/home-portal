use std::collections::BTreeMap;

use portal_feature::{Action, ApiError, Area, FieldError, Principal, Right, Rights};
use toml_edit::DocumentMut;

use super::groups::{name_problem, rights_of_group};
use super::user_listing::users_of;
use crate::types::{GroupView, GroupsView, UsersSection};

pub const ADMIN_GROUPS: &str = "only members of admin may change groups";
pub const NAME_FIELD: &str = "name";

pub fn groups_view(document: &DocumentMut) -> Result<GroupsView, ApiError> {
    let section = users_of(document)?;
    let members = |group: &str| -> Vec<String> {
        section
            .users
            .iter()
            .filter(|user| user.group.as_deref() == Some(group))
            .map(|user| user.name.clone())
            .collect()
    };
    let mut groups = vec![GroupView {
        name: Principal::ADMIN.to_string(),
        builtin: true,
        rights: Rights::admin(),
        members: members(Principal::ADMIN),
    }];
    groups.extend(section.groups.iter().map(|group| GroupView {
        name: group.name.clone(),
        builtin: false,
        rights: rights_of_group(group),
        members: members(&group.name),
    }));
    Ok(GroupsView { groups })
}

pub fn require_admin(actor: &Principal) -> Result<(), ApiError> {
    if actor.rights.is_admin() {
        Ok(())
    } else {
        Err(ApiError::Forbidden(ADMIN_GROUPS.to_string()))
    }
}

pub fn checked_group_name(
    name: &str,
    section: &UsersSection,
    current: Option<&str>,
) -> Result<String, FieldError> {
    let trimmed = name.trim();
    if let Some(problem) = name_problem(trimmed) {
        return Err(FieldError::new(NAME_FIELD, problem));
    }
    if trimmed == Principal::ADMIN {
        return Err(FieldError::new(NAME_FIELD, "admin is built in"));
    }
    let taken = section
        .groups
        .iter()
        .any(|group| group.name == trimmed && Some(group.name.as_str()) != current);
    if taken {
        return Err(FieldError::new(NAME_FIELD, "is used by another group"));
    }
    Ok(trimmed.to_string())
}

pub fn checked_rights(rights: &BTreeMap<String, Vec<String>>) -> Result<Rights, Vec<FieldError>> {
    let mut granted = Vec::new();
    let mut errors = Vec::new();
    for (area_name, actions) in rights {
        let field = format!("rights.{area_name}");
        let Some(area) = Area::from_name(area_name) else {
            errors.push(FieldError::new(field, "is not an area"));
            continue;
        };
        for action_name in actions {
            match Action::from_name(action_name).filter(|action| area.accepts(*action)) {
                Some(action) => granted.push(Right::new(area, action)),
                None => {
                    let accepted: Vec<&str> =
                        area.actions().iter().map(|action| action.name()).collect();
                    errors.push(FieldError::new(
                        field.clone(),
                        format!("accepts only {}", accepted.join(", ")),
                    ));
                    break;
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(Rights::of(granted))
    } else {
        Err(errors)
    }
}
