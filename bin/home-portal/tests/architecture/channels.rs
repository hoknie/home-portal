use std::collections::BTreeSet;
use std::fs;

use home_portal::channels;

use crate::sources::workspace_root;

pub const CHANNELS_FOLDER: &str = "crates/notification";
pub const CORE_FOLDER: &str = "crates/core";
pub const CRATE_PREFIX: &str = "portal-";

fn crates_in(folder: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(workspace_root().join(folder))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_dir())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

pub fn differences(on_disk: &[String], registered: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    for name in registered {
        if !seen.insert(*name) {
            found.push(format!("channel {name} is registered twice"));
        }
    }
    for name in on_disk {
        if !seen.contains(name.as_str()) {
            found.push(format!("channel {name} has a crate and is not registered"));
        }
    }
    for name in &seen {
        if !on_disk.iter().any(|disk| disk == name) {
            found.push(format!("channel {name} is registered and has no crate"));
        }
    }
    found
}

pub fn forbidden_dependencies(channel: &str, manifest: &str, core: &[String]) -> Vec<String> {
    let mut in_dependencies = false;
    let mut found = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        let Some(name) = line.split(['.', '=', ' ']).next() else {
            continue;
        };
        if in_dependencies
            && name.starts_with(CRATE_PREFIX)
            && !core.iter().any(|allowed| allowed == name)
        {
            found.push(format!(
                "channel {channel} depends on {name}, which is not a core crate"
            ));
        }
    }
    found
}

#[test]
fn the_channel_list_names_exactly_the_channel_crates_on_disk() {
    let registered = channels("http://127.0.0.1:9").unwrap();
    let names: Vec<&str> = registered.iter().map(|channel| channel.name()).collect();
    let on_disk: Vec<String> = crates_in(CHANNELS_FOLDER)
        .into_iter()
        .filter_map(|name| name.strip_prefix(CRATE_PREFIX).map(str::to_string))
        .collect();
    let found = differences(&on_disk, &names);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn channel_crates_depend_only_on_core_crates() {
    let core = crates_in(CORE_FOLDER);
    let mut found = Vec::new();
    for name in crates_in(CHANNELS_FOLDER) {
        let manifest = fs::read_to_string(
            workspace_root()
                .join(CHANNELS_FOLDER)
                .join(&name)
                .join("Cargo.toml"),
        )
        .unwrap();
        found.extend(forbidden_dependencies(&name, &manifest, &core));
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_forgotten_channel_crate_is_named() {
    let disk = vec!["telegram".to_string(), "mail".to_string()];
    assert_eq!(
        differences(&disk, &["telegram"]),
        vec!["channel mail has a crate and is not registered"]
    );
}

#[test]
fn a_channel_that_reaches_into_a_feature_is_named() {
    let manifest = "[package]\nname = \"portal-telegram\"\n\n[dependencies]\nportal-feature.workspace = true\nportal-services.workspace = true\n\n[dev-dependencies]\nportal-dns.workspace = true\n";
    let core = vec!["portal-feature".to_string(), "portal-config".to_string()];
    assert_eq!(
        forbidden_dependencies("portal-telegram", manifest, &core),
        vec!["channel portal-telegram depends on portal-services, which is not a core crate"]
    );
}
