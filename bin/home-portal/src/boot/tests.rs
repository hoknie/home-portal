use std::fs;

use portal_config::ConfigStore;

use super::address::ADDRESS_VARIABLE;
use super::interface::interface_folder;
use super::shutdown::requested;
use super::start::replaced_executable;
use super::{parse_address, resolve_address};
use crate::types::{BootError, Ended, Restart};

fn store(text: &str) -> (tempfile::TempDir, ConfigStore) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let store = ConfigStore::open(&path).unwrap();
    (directory, store)
}

#[test]
fn without_a_network_section_or_the_variable_the_portal_listens_on_the_default_address() {
    let (_directory, store) = store("");
    let effective = resolve_address(&store, None).unwrap();
    assert_eq!(effective.address.to_string(), "127.0.0.1:8080");
    assert!(!effective.overridden);
}

#[test]
fn the_configuration_chooses_the_address() {
    let (_directory, store) = store("[network]\naddress = \"127.0.0.1\"\nport = 9191\n");
    assert_eq!(
        resolve_address(&store, None).unwrap().address.to_string(),
        "127.0.0.1:9191"
    );
}

#[test]
fn the_variable_overrides_the_configuration() {
    let (_directory, store) = store("[network]\nport = 9191\n");
    let effective = resolve_address(&store, Some("127.0.0.1:9090".into())).unwrap();
    assert_eq!(effective.address.to_string(), "127.0.0.1:9090");
    assert!(effective.overridden);
}

#[test]
fn an_address_that_does_not_parse_is_refused_by_name() {
    let error = parse_address("not-an-address".into()).unwrap_err();
    assert!(matches!(error, BootError::Address { .. }));
    let message = error.to_string();
    assert!(message.contains("not-an-address"));
    assert!(message.contains(ADDRESS_VARIABLE));
}

#[test]
fn a_port_out_of_range_in_the_file_is_refused_by_key() {
    let (_directory, store) = store("[network]\nport = 70000\n");
    let message = resolve_address(&store, None).unwrap_err().to_string();
    assert!(message.contains("network.port"), "{message}");
}

#[test]
fn a_replaced_executable_is_found_at_its_path() {
    let directory = tempfile::tempdir().unwrap();
    let binary = directory.path().join("home-portal");
    fs::write(&binary, "").unwrap();
    let deleted = directory.path().join("home-portal (deleted)");
    assert_eq!(replaced_executable(deleted), binary);
    let gone = directory.path().join("gone (deleted)");
    assert_eq!(replaced_executable(gone.clone()), gone);
    assert_eq!(replaced_executable(binary.clone()), binary);
}

#[tokio::test]
async fn a_restart_request_ends_the_serving_with_a_restart_and_a_second_is_harmless() {
    let restart = Restart::default();
    assert!(!restart.requested());
    let waiting = tokio::spawn(requested(restart.clone()));
    restart.request();
    restart.request();
    let ended = tokio::time::timeout(std::time::Duration::from_secs(2), waiting)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ended, Ended::Restart);
    assert!(restart.requested());
    assert_eq!(requested(restart).await, Ended::Restart);
}

#[test]
fn the_interface_is_looked_for_in_the_variable_then_beside_the_binary_then_in_share() {
    let prefix = tempfile::tempdir().unwrap();
    let bin = prefix.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let binary = bin.join("home-portal");
    fs::write(&binary, "").unwrap();
    let link_folder = tempfile::tempdir().unwrap();
    let link = link_folder.path().join("home-portal");
    std::os::unix::fs::symlink(&binary, &link).unwrap();
    let real = fs::canonicalize(prefix.path()).unwrap();
    assert_eq!(
        interface_folder(None, Some(link)),
        vec![real.join("bin/web"), real.join("share/home-portal/web")]
    );
    assert_eq!(
        interface_folder(Some("/srv/web".into()), Some(binary.clone())),
        vec![std::path::PathBuf::from("/srv/web")]
    );
    assert_eq!(interface_folder(Some("".into()), Some(binary)).len(), 2);
    assert!(interface_folder(None, None).is_empty());
}
