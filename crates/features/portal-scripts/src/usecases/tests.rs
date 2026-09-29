use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::Arc;

use portal_config::ConfigStore;

use super::{ListScripts, ScriptEditing};
use crate::fakes::FileOwner;

#[test]
fn the_listing_reads_each_header_and_never_follows_a_link_out() {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        root.join("restart.sh"),
        "#!/bin/sh\n# @description Restart\n# @arg service\n",
    )
    .unwrap();
    fs::set_permissions(root.join("restart.sh"), fs::Permissions::from_mode(0o644)).unwrap();
    let outside = folder.path().join("secret.sh");
    fs::write(&outside, "# @description Private\n").unwrap();
    symlink(&outside, root.join("link.sh")).unwrap();
    let listing = ListScripts::at(root, Arc::new(FileOwner::of_this_process()));
    let files = listing.run().unwrap();
    let restart = files.iter().find(|file| file.path == "restart.sh").unwrap();
    assert!(restart.problem.is_some());
    assert_eq!(restart.header.description.as_deref(), Some("Restart"));
    assert_eq!(restart.header.arguments[0].name, "service");
    let link = files.iter().find(|file| file.path == "link.sh").unwrap();
    assert_eq!(link.header.description, None);
}

#[test]
fn a_changed_header_is_read_again() {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    let script = root.join("a.sh");
    fs::write(&script, "# @arg one\n").unwrap();
    let listing = ListScripts::at(root, Arc::new(FileOwner::of_this_process()));
    assert_eq!(listing.run().unwrap()[0].header.arguments.len(), 1);
    fs::write(&script, "# @arg one\n# @arg two?\n").unwrap();
    assert_eq!(listing.run().unwrap()[0].header.arguments.len(), 2);
}

#[test]
fn editing_follows_the_file_without_a_restart() {
    let folder = tempfile::tempdir().unwrap();
    let main = folder.path().join("home-portal.toml");
    fs::write(&main, "").unwrap();
    let editing = ScriptEditing::new(Arc::new(ConfigStore::open(&main).unwrap()));
    assert!(!editing.run());
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(&main, "[scripts]\nediting = true\n").unwrap();
    assert!(editing.run());
}
