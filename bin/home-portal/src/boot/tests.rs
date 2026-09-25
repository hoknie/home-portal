use std::fs;

use portal_config::ConfigStore;

use super::address::ADDRESS_VARIABLE;
use super::{parse_address, resolve_address};
use crate::types::BootError;

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
