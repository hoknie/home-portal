use std::collections::BTreeSet;
use std::fs;

use home_portal::registered;

use crate::sources::workspace_root;

pub const FEATURES_FOLDER: &str = "crates/features";
pub const CRATE_PREFIX: &str = "portal-";

pub fn on_disk() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(workspace_root().join(FEATURES_FOLDER))
        .expect("the features folder exists")
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.strip_prefix(CRATE_PREFIX).map(str::to_string))
        .collect();
    names.sort();
    names
}

pub fn differences(on_disk: &[String], registered: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    for name in registered {
        if !seen.insert(*name) {
            found.push(format!("feature {name} is registered twice"));
        }
    }
    for name in on_disk {
        if !seen.contains(name.as_str()) {
            found.push(format!("feature {name} has a crate and is not registered"));
        }
    }
    for name in &seen {
        if !on_disk.iter().any(|disk| disk == name) {
            found.push(format!("feature {name} is registered and has no crate"));
        }
    }
    found
}

#[test]
fn the_registry_names_exactly_the_feature_crates_on_disk() {
    let directory = tempfile::tempdir().unwrap();
    let path = crate::support::with_extra(&directory, "secret", "");
    let features = registered(&crate::support::wiring_for(&path))
        .unwrap()
        .features;
    let names: Vec<&str> = features.iter().map(|feature| feature.name()).collect();
    let found = differences(&on_disk(), &names);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_forgotten_crate_is_named() {
    let disk = vec!["health".to_string(), "probe".to_string()];
    assert_eq!(
        differences(&disk, &["health"]),
        vec!["feature probe has a crate and is not registered"]
    );
}

#[test]
fn a_registered_feature_without_a_crate_is_named() {
    let disk = vec!["health".to_string()];
    assert_eq!(
        differences(&disk, &["health", "ghost"]),
        vec!["feature ghost is registered and has no crate"]
    );
}

#[test]
fn a_duplicate_name_is_named() {
    let disk = vec!["health".to_string()];
    assert_eq!(
        differences(&disk, &["health", "health"]),
        vec!["feature health is registered twice"]
    );
}
