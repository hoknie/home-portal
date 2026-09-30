use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tempfile::TempDir;

use super::automation::{AutomationCheck, refusal};
use super::folders::FolderCheck;
use super::full_disk::FullDiskCheck;
use super::local_network::{LocalNetworkCheck, Sender, query};
use super::volumes::{VolumesCheck, removable_in};
use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, PermissionState};

fn locked(path: &Path) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o000)).unwrap();
}

fn unlocked(path: &Path) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn full_disk_access_is_read_from_whether_the_protected_file_opens() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("TCC.db");
    fs::write(&path, "").unwrap();
    let check = FullDiskCheck { path: path.clone() };
    assert_eq!(check.ask(), Finding::granted());
    locked(&path);
    assert_eq!(check.ask(), Finding::denied(Advice::GrantFullDiskAccess));
    let missing = FullDiskCheck {
        path: folder.path().join("missing.db"),
    };
    assert_eq!(missing.ask().state, PermissionState::Unknown);
}

#[test]
fn a_folder_is_granted_when_it_lists_and_denied_when_it_refuses() {
    let folder = TempDir::new().unwrap();
    let documents = folder.path().join("Documents");
    fs::create_dir(&documents).unwrap();
    let check = FolderCheck {
        name: "Documents".to_string(),
        path: documents.clone(),
    };
    assert_eq!(check.ask(), Finding::granted());
    locked(&documents);
    assert_eq!(check.ask(), Finding::denied(Advice::AllowInSettings));
    unlocked(&documents);
    let missing = FolderCheck {
        name: "Desktop".to_string(),
        path: folder.path().join("Desktop"),
    };
    assert_eq!(missing.ask().state, PermissionState::NotApplicable);
}

fn volumes(root: &Path) -> VolumesCheck {
    VolumesCheck {
        root: root.to_path_buf(),
        system_device: None,
        removable: Arc::new(|path: &Path| path.ends_with("USB")),
    }
}

#[test]
fn without_a_removable_volume_the_advice_is_to_connect_one() {
    let root = TempDir::new().unwrap();
    fs::create_dir(root.path().join("Internal")).unwrap();
    let finding = volumes(root.path()).ask();
    assert_eq!(finding.state, PermissionState::NotApplicable);
    assert_eq!(finding.advice, Some(Advice::ConnectAVolume));
}

#[test]
fn a_removable_volume_that_refuses_to_list_is_denied() {
    let root = TempDir::new().unwrap();
    let usb = root.path().join("USB");
    fs::create_dir(&usb).unwrap();
    fs::create_dir(root.path().join("Internal")).unwrap();
    assert_eq!(volumes(root.path()).ask(), Finding::granted());
    locked(&usb);
    assert_eq!(
        volumes(root.path()).ask(),
        Finding::denied(Advice::AllowInSettings)
    );
    unlocked(&usb);
}

#[test]
fn a_volume_on_the_system_device_is_skipped() {
    let root = TempDir::new().unwrap();
    let usb = root.path().join("USB");
    fs::create_dir(&usb).unwrap();
    let mut check = volumes(root.path());
    check.system_device = super::volumes::system_device_of(&usb);
    assert_eq!(check.ask().state, PermissionState::NotApplicable);
}

#[test]
fn diskutil_says_which_volumes_are_removable() {
    let external =
        "<dict>\n\t<key>Ejectable</key>\n\t<false/>\n\t<key>Internal</key>\n\t<false/>\n</dict>";
    let internal = "<dict><key>Ejectable</key><false/><key>Internal</key><true/><key>RemovableMedia</key><false/></dict>";
    let card = "<dict><key>Internal</key><true/><key>RemovableMedia</key><true/></dict>";
    assert!(removable_in(external));
    assert!(!removable_in(internal));
    assert!(removable_in(card));
    assert!(!removable_in(""));
}

#[test]
fn the_local_network_query_asks_one_pointer_question() {
    let bytes = query();
    assert_eq!(&bytes[..12], &[0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(bytes[12], 12);
    assert_eq!(&bytes[13..25], b"_home-portal");
    assert_eq!(&bytes[bytes.len() - 5..], &[0, 0, 12, 0, 1]);
}

fn network(send: Sender, limit: Duration) -> LocalNetworkCheck {
    LocalNetworkCheck {
        send,
        limit,
        retry: Duration::from_millis(1),
    }
}

fn failing(kind: ErrorKind) -> Sender {
    Arc::new(move |_: &[u8]| Err(io::Error::from(kind)))
}

#[test]
fn the_local_network_is_granted_when_a_datagram_leaves() {
    let check = network(Arc::new(|_: &[u8]| Ok(())), Duration::ZERO);
    assert_eq!(check.ask(), Finding::granted());
}

#[test]
fn an_unreachable_local_network_past_the_limit_is_denied() {
    let check = network(failing(ErrorKind::HostUnreachable), Duration::ZERO);
    assert_eq!(check.ask(), Finding::denied(Advice::AllowInSettings));
}

#[test]
fn an_answer_within_the_limit_turns_the_local_network_granted() {
    let tries = Arc::new(AtomicUsize::new(0));
    let counted = tries.clone();
    let send: Sender = Arc::new(move |_: &[u8]| {
        if counted.fetch_add(1, Ordering::SeqCst) < 2 {
            Err(io::Error::from(ErrorKind::HostUnreachable))
        } else {
            Ok(())
        }
    });
    let check = network(send, Duration::from_secs(5));
    assert_eq!(check.ask(), Finding::granted());
    assert_eq!(tries.load(Ordering::SeqCst), 3);
}

#[test]
fn without_a_network_the_local_network_does_not_apply() {
    let check = network(failing(ErrorKind::NetworkUnreachable), Duration::ZERO);
    assert_eq!(check.ask().state, PermissionState::NotApplicable);
}

fn stub(folder: &TempDir, body: &str) -> PathBuf {
    let path = folder.path().join("osascript");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn automation(program: PathBuf, limit: Duration) -> AutomationCheck {
    AutomationCheck {
        application: "Finder".to_string(),
        program,
        limit,
    }
}

#[test]
fn automation_passes_the_application_as_the_last_argument() {
    let folder = TempDir::new().unwrap();
    let program = stub(&folder, "[ \"$7\" = Finder ] || exit 3\nexit 0");
    let check = automation(program, Duration::from_secs(5));
    assert_eq!(check.ask(), Finding::granted());
}

#[test]
fn automation_errors_are_read_from_their_codes() {
    let folder = TempDir::new().unwrap();
    let program = stub(
        &folder,
        "echo 'execution error: Not authorized to send Apple events to Finder. (-1743)' >&2\nexit 1",
    );
    let check = automation(program, Duration::from_secs(5));
    assert_eq!(check.ask(), Finding::denied(Advice::AllowInSettings));
    assert_eq!(
        refusal("execution error: Application isn't running. (-600)"),
        Finding::unknown(Advice::ApplicationNotFound)
    );
    assert_eq!(
        refusal("syntax error"),
        Finding::unknown(Advice::CheckFailed)
    );
}

#[test]
fn an_unanswered_automation_prompt_is_stopped_at_the_limit() {
    let folder = TempDir::new().unwrap();
    let program = stub(&folder, "exec sleep 30");
    let check = automation(program, Duration::from_millis(200));
    assert_eq!(check.ask(), Finding::unknown(Advice::CheckFailed));
}
