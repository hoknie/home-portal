use portal_feature::{Action, ApiError, Area, FieldError, Principal, Right};

use super::groups::rights_of_group;
use crate::types::UsersSection;

pub const ADMIN_ONLY: &str =
    "only members of admin may change a member of admin or put someone into admin";
pub const TOO_MUCH: &str = "you may give only a group whose rights are all within your own";
pub const LAST_ADMIN: &str = "at least one user must stay in the admin group";
pub const UNKNOWN_GROUP: &str = "names no group";
pub const GROUP_FIELD: &str = "group";
pub const CHANGE_USERS: Right = Right::new(Area::Users, Action::Update);

pub fn in_admin(section: &UsersSection, name: &str) -> bool {
    section
        .find(name)
        .is_some_and(|user| user.group.as_deref() == Some(Principal::ADMIN))
}

pub fn admins(section: &UsersSection) -> usize {
    section
        .users
        .iter()
        .filter(|user| user.group.as_deref() == Some(Principal::ADMIN))
        .count()
}

pub fn may_touch(actor: &Principal, section: &UsersSection, target: &str) -> Result<(), ApiError> {
    if in_admin(section, target) && !actor.rights.is_admin() {
        return Err(ApiError::Forbidden(ADMIN_ONLY.to_string()));
    }
    Ok(())
}

pub fn unknown_group(section: &UsersSection, group: Option<&str>) -> Option<FieldError> {
    let name = group?;
    (name != Principal::ADMIN && section.group(name).is_none())
        .then(|| FieldError::new(GROUP_FIELD, UNKNOWN_GROUP))
}

pub fn may_give(
    actor: &Principal,
    section: &UsersSection,
    group: Option<&str>,
) -> Result<(), ApiError> {
    if actor.rights.is_admin() {
        return Ok(());
    }
    match group {
        None => Ok(()),
        Some(Principal::ADMIN) => Err(ApiError::Forbidden(ADMIN_ONLY.to_string())),
        Some(name) => {
            let within = section
                .group(name)
                .is_none_or(|group| rights_of_group(group).within(&actor.rights));
            if within {
                Ok(())
            } else {
                Err(ApiError::Forbidden(TOO_MUCH.to_string()))
            }
        }
    }
}

pub fn keeps_an_admin(
    section: &UsersSection,
    name: &str,
    next: Option<&str>,
) -> Result<(), ApiError> {
    let leaving = in_admin(section, name) && next != Some(Principal::ADMIN);
    if leaving && admins(section) <= 1 {
        return Err(ApiError::Conflict(LAST_ADMIN.to_string()));
    }
    Ok(())
}
