use crate::types::{PermissionCode, PermissionSettings};

pub fn codes_of(settings: &PermissionSettings) -> Vec<PermissionCode> {
    let mut codes = vec![
        PermissionCode::LocalNetwork,
        PermissionCode::RemovableVolumes,
    ];
    codes.extend(settings.folders.iter().cloned().map(PermissionCode::Folder));
    codes.extend(
        settings
            .automation
            .iter()
            .cloned()
            .map(PermissionCode::Automation),
    );
    codes.push(PermissionCode::FullDiskAccess);
    codes
}
