use std::path::PathBuf;

use super::{Owner, OwnerKind, Pane, PermissionCode, PermissionSettings};

#[test]
fn a_terminal_owns_the_permissions_of_a_process_started_in_it() {
    let owner = Owner::detect(
        Some("Apple_Terminal".to_string()),
        Some(PathBuf::from("/usr/local/bin/home-portal")),
    );
    assert_eq!(owner.kind, OwnerKind::Terminal);
    assert_eq!(owner.name, "Terminal");
}

#[test]
fn without_a_terminal_the_binary_owns_its_permissions() {
    let owner = Owner::detect(None, Some(PathBuf::from("/usr/local/bin/home-portal")));
    assert_eq!(owner.kind, OwnerKind::Binary);
    assert_eq!(owner.name, "/usr/local/bin/home-portal");
    let empty = Owner::detect(Some(String::new()), None);
    assert_eq!(empty.kind, OwnerKind::Binary);
    assert_eq!(empty.name, Owner::SOME_BINARY);
}

#[test]
fn codes_name_their_folder_or_application_and_their_pane() {
    let folder = PermissionCode::Folder("Documents".to_string());
    assert_eq!(folder.code(), "folder:Documents");
    assert_eq!(folder.pane(), Pane::FilesAndFolders);
    let automation = PermissionCode::Automation("System Events".to_string());
    assert_eq!(automation.code(), "automation:System Events");
    assert_eq!(automation.pane(), Pane::Automation);
    assert!(!PermissionCode::FullDiskAccess.requestable());
}

#[test]
fn the_defaults_ask_at_start_for_system_events_and_three_folders() {
    let settings = PermissionSettings::default();
    assert!(settings.request_at_start);
    assert_eq!(settings.automation, vec!["System Events"]);
    assert_eq!(settings.folders, vec!["Documents", "Desktop", "Downloads"]);
}
