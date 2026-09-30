use super::{Owner, PermissionView};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionsView {
    pub platform: &'static str,
    pub owner: Owner,
    pub permissions: Vec<PermissionView>,
}

impl PermissionsView {
    pub fn denied(&self) -> bool {
        self.permissions
            .iter()
            .any(|permission| permission.state == super::PermissionState::Denied)
    }
}
