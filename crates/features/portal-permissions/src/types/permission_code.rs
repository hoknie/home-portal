use super::Pane;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PermissionCode {
    LocalNetwork,
    RemovableVolumes,
    Folder(String),
    Automation(String),
    FullDiskAccess,
}

impl PermissionCode {
    pub const LOCAL_NETWORK: &'static str = "local-network";
    pub const REMOVABLE_VOLUMES: &'static str = "removable-volumes";
    pub const FOLDER: &'static str = "folder";
    pub const AUTOMATION: &'static str = "automation";
    pub const FULL_DISK_ACCESS: &'static str = "full-disk-access";

    pub fn code(&self) -> String {
        match self {
            PermissionCode::LocalNetwork => Self::LOCAL_NETWORK.to_string(),
            PermissionCode::RemovableVolumes => Self::REMOVABLE_VOLUMES.to_string(),
            PermissionCode::Folder(name) => format!("{}:{name}", Self::FOLDER),
            PermissionCode::Automation(application) => {
                format!("{}:{application}", Self::AUTOMATION)
            }
            PermissionCode::FullDiskAccess => Self::FULL_DISK_ACCESS.to_string(),
        }
    }

    pub fn pane(&self) -> Pane {
        match self {
            PermissionCode::LocalNetwork => Pane::LocalNetwork,
            PermissionCode::RemovableVolumes | PermissionCode::Folder(_) => Pane::FilesAndFolders,
            PermissionCode::Automation(_) => Pane::Automation,
            PermissionCode::FullDiskAccess => Pane::FullDiskAccess,
        }
    }

    pub fn requestable(&self) -> bool {
        !matches!(self, PermissionCode::FullDiskAccess)
    }
}
