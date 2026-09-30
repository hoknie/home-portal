use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionState {
    Granted,
    Denied,
    Pending,
    NotApplicable,
    Unknown,
}

impl PermissionState {
    pub fn name(self) -> &'static str {
        match self {
            PermissionState::Granted => "granted",
            PermissionState::Denied => "denied",
            PermissionState::Pending => "pending",
            PermissionState::NotApplicable => "not-applicable",
            PermissionState::Unknown => "unknown",
        }
    }

    pub fn settled(self) -> bool {
        matches!(
            self,
            PermissionState::Granted | PermissionState::NotApplicable
        )
    }
}
