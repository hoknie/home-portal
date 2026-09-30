use std::collections::HashSet;

use portal_feature::{Action, Area, FieldError, Principal, Right, Rights};

use crate::types::{Group, User, UsersSection};

pub const NO_ADMIN: &str =
    "no user is in the admin group; add group = \"admin\" to the owner's [[users]] entry";
pub const LONGEST_NAME: usize = 64;

pub fn rights_of_group(group: &Group) -> Rights {
    Rights::of(group.permissions.iter().flat_map(|(area, actions)| {
        let area = Area::from_name(area);
        actions
            .iter()
            .filter_map(move |action| Some(Right::new(area?, Action::from_name(action)?)))
    }))
}

pub fn rights_for(section: &UsersSection, user: &User) -> Rights {
    match user.group.as_deref() {
        Some(Principal::ADMIN) => Rights::admin(),
        Some(name) => section.group(name).map(rights_of_group).unwrap_or_default(),
        None => Rights::none(),
    }
}

pub fn principal_for(section: &UsersSection, name: &str) -> Principal {
    let user = section.find(name);
    let group = user.and_then(|user| user.group.clone());
    let rights = user.map_or_else(Rights::none, |user| rights_for(section, user));
    Principal::member(name, group, rights)
}

pub fn group_errors(section: &UsersSection) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (index, group) in section.groups.iter().enumerate() {
        let field = format!("groups[{index}].name");
        if let Some(problem) = name_problem(&group.name) {
            errors.push(FieldError::new(field, problem));
        } else if group.name == Principal::ADMIN {
            errors.push(FieldError::new(
                field,
                "admin is built in with every right and cannot be declared",
            ));
        } else if !seen.insert(group.name.as_str()) {
            errors.push(FieldError::new(field, "is used by another group"));
        }
        errors.extend(permission_errors(index, group));
    }
    for (index, user) in section.users.iter().enumerate() {
        let Some(name) = user.group.as_deref() else {
            continue;
        };
        if name != Principal::ADMIN && section.group(name).is_none() {
            errors.push(FieldError::new(
                format!("users[{index}].group"),
                format!("names no group; declare [[groups]] name = \"{name}\" or use \"admin\""),
            ));
        }
    }
    let admins = section
        .users
        .iter()
        .filter(|user| user.group.as_deref() == Some(Principal::ADMIN))
        .count();
    if admins == 0 {
        errors.push(FieldError::new("users", NO_ADMIN));
    }
    errors
}

pub fn name_problem(name: &str) -> Option<&'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Some("must not be empty")
    } else if trimmed.chars().count() > LONGEST_NAME {
        Some("must be at most 64 characters")
    } else if trimmed.chars().any(char::is_control) {
        Some("must not contain control characters")
    } else {
        None
    }
}

pub fn permission_errors(index: usize, group: &Group) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for (area_name, actions) in &group.permissions {
        let field = format!("groups[{index}].permissions.{area_name}");
        let Some(area) = Area::from_name(area_name) else {
            let known: Vec<&str> = Area::ALL.iter().map(|area| area.name()).collect();
            errors.push(FieldError::new(
                field,
                format!("is not an area; the areas are {}", known.join(", ")),
            ));
            continue;
        };
        let accepted: Vec<&str> = area.actions().iter().map(|action| action.name()).collect();
        if actions
            .iter()
            .any(|action| Action::from_name(action).is_none_or(|action| !area.accepts(action)))
        {
            errors.push(FieldError::new(
                field,
                format!("accepts only {}", accepted.join(", ")),
            ));
        }
    }
    errors
}
