use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use toml_edit::DocumentMut;

use super::{HostChecks, PermissionBoard, asked, codes_of, read_settings, validate_permissions};
use crate::ports::{CheckSource, PermissionCheck};
use crate::types::{
    Advice, Finding, Limits, Owner, PermissionCode, PermissionSettings, PermissionState,
};

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

fn fields(text: &str) -> Vec<String> {
    validate_permissions(&document(text))
        .into_iter()
        .map(|error| error.field)
        .collect()
}

#[test]
fn without_a_table_the_defaults_apply() {
    assert_eq!(read_settings(&document("")), PermissionSettings::default());
    assert!(fields("").is_empty());
}

#[test]
fn automation_for_more_applications_is_read_in_order() {
    let settings = read_settings(&document(
        "[permissions]\nrequest_at_start = false\nautomation = [\"System Events\", \"Finder\"]\n",
    ));
    assert!(!settings.request_at_start);
    assert_eq!(settings.automation, vec!["System Events", "Finder"]);
    assert_eq!(settings.folders, PermissionSettings::default().folders);
}

#[test]
fn empty_lists_ask_for_nothing_of_their_kind() {
    let settings = read_settings(&document("[permissions]\nautomation = []\nfolders = []\n"));
    assert!(settings.automation.is_empty());
    assert!(settings.folders.is_empty());
    assert_eq!(
        codes_of(&settings),
        vec![
            PermissionCode::LocalNetwork,
            PermissionCode::RemovableVolumes,
            PermissionCode::FullDiskAccess
        ]
    );
}

#[test]
fn an_unknown_folder_names_permissions_folders() {
    assert_eq!(
        fields("[permissions]\nfolders = [\"Pictures\"]\n"),
        vec!["permissions.folders"]
    );
}

#[test]
fn an_empty_application_name_names_permissions_automation() {
    assert_eq!(
        fields("[permissions]\nautomation = [\" \"]\n"),
        vec!["permissions.automation"]
    );
}

#[test]
fn a_wrong_type_or_an_unknown_key_is_named() {
    assert_eq!(
        fields("[permissions]\nrequest_at_start = \"yes\"\ncamera = true\n"),
        vec!["permissions.request_at_start", "permissions.camera"]
    );
    assert_eq!(fields("permissions = 1\n"), vec!["permissions"]);
}

#[test]
fn codes_list_folders_and_applications_between_volumes_and_full_disk_access() {
    let codes: Vec<String> = codes_of(&PermissionSettings::default())
        .iter()
        .map(PermissionCode::code)
        .collect();
    assert_eq!(
        codes,
        vec![
            "local-network",
            "removable-volumes",
            "folder:Documents",
            "folder:Desktop",
            "folder:Downloads",
            "automation:System Events",
            "full-disk-access"
        ]
    );
}

#[test]
fn on_another_system_every_check_does_not_apply() {
    let host = HostChecks {
        macos: false,
        home: Some(PathBuf::from("/home/anna")),
        limits: Limits::default(),
    };
    assert!(!host.applies());
    let findings: Vec<PermissionState> = host
        .checks(&PermissionSettings::default())
        .iter()
        .map(|check| check.ask().state)
        .collect();
    assert_eq!(findings, vec![PermissionState::NotApplicable; 7]);
}

struct Held {
    answer: Mutex<Receiver<Finding>>,
}

impl PermissionCheck for Held {
    fn code(&self) -> PermissionCode {
        PermissionCode::Folder("Documents".to_string())
    }

    fn ask(&self) -> Finding {
        self.answer
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .recv()
            .unwrap_or(Finding::unknown(Advice::CheckFailed))
    }
}

async fn eventually(board: &PermissionBoard, code: &PermissionCode, state: PermissionState) {
    for _ in 0..200 {
        if board.get(code).map(|(finding, _)| finding.state) == Some(state) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("{code:?} never became {state:?}");
}

#[tokio::test]
async fn a_read_blocked_past_the_limit_is_pending_until_it_answers() {
    let (answer, waiting) = channel();
    let check = Arc::new(Held {
        answer: Mutex::new(waiting),
    });
    let code = check.code();
    let board = PermissionBoard::default();
    let owner = Owner::detect(None, None);
    asked(
        check.clone(),
        board.clone(),
        Duration::from_millis(50),
        owner.clone(),
    )
    .await;
    assert_eq!(board.get(&code).unwrap().0, Finding::pending());
    assert!(board.asking(&code));
    asked(check, board.clone(), Duration::from_millis(50), owner).await;
    assert!(board.asking(&code));
    answer.send(Finding::granted()).unwrap();
    eventually(&board, &code, PermissionState::Granted).await;
    assert!(!board.asking(&code));
}
