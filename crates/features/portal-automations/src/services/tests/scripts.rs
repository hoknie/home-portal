use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

use tempfile::TempDir;

use crate::services::ScriptsDirectory;

fn directory() -> (TempDir, ScriptsDirectory) {
    let folder = TempDir::new().unwrap();
    let root = folder.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    let directory = ScriptsDirectory::at(folder.path().join("scripts"));
    (folder, directory)
}

fn file(root: &Path, name: &str, mode: u32) {
    let path = root.join(name);
    fs::write(&path, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
}

fn refusal(directory: &ScriptsDirectory, name: &str) -> String {
    directory.resolve(name).unwrap_err().message
}

#[test]
fn a_private_executable_inside_the_directory_is_accepted() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o755);
    assert!(directory.resolve("backup.sh").is_ok());
}

#[test]
fn a_link_that_leaves_the_directory_is_refused_as_outside() {
    let (_folder, directory) = directory();
    symlink("/bin/sh", directory.root().join("evil.sh")).unwrap();
    assert!(refusal(&directory, "evil.sh").contains("outside the scripts directory"));
}

#[test]
fn a_script_anyone_can_write_is_refused() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o777);
    assert!(refusal(&directory, "backup.sh").contains("written by group or others"));
}

#[test]
fn a_private_script_in_a_directory_anyone_can_write_is_refused() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o755);
    fs::set_permissions(directory.root(), fs::Permissions::from_mode(0o777)).unwrap();
    assert!(refusal(&directory, "backup.sh").contains("which group or others can write"));
}

#[test]
fn a_script_that_is_not_executable_is_refused() {
    let (_folder, directory) = directory();
    file(directory.root(), "notes.sh", 0o644);
    assert!(refusal(&directory, "notes.sh").contains("not executable"));
}

#[test]
fn a_directory_is_refused() {
    let (_folder, directory) = directory();
    fs::create_dir(directory.root().join("media")).unwrap();
    assert!(refusal(&directory, "media").contains("not a regular file"));
}

#[test]
fn the_listing_goes_two_levels_deep_and_says_why_a_script_cannot_run() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o755);
    file(directory.root(), "open.sh", 0o777);
    let nested = directory.root().join("media");
    fs::create_dir(&nested).unwrap();
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o755)).unwrap();
    file(&nested, "restart.sh", 0o755);
    let deeper = nested.join("deeper");
    fs::create_dir(&deeper).unwrap();
    file(&deeper, "hidden.sh", 0o755);
    let listed = directory.list().unwrap();
    let names: Vec<&str> = listed.iter().map(|entry| entry.path.as_str()).collect();
    assert_eq!(names, vec!["backup.sh", "media/restart.sh", "open.sh"]);
    assert!(listed[0].problem.is_none());
    assert!(listed[2].problem.is_some());
}

#[test]
fn a_missing_directory_is_no_listing() {
    let folder = TempDir::new().unwrap();
    let directory = ScriptsDirectory::at(folder.path().join("scripts"));
    assert!(directory.list().is_none());
}

#[test]
fn a_script_deeper_than_one_subfolder_or_hidden_is_refused_and_not_listed() {
    let (_folder, directory) = directory();
    let nested = directory.root().join("media");
    fs::create_dir(&nested).unwrap();
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o755)).unwrap();
    file(&nested, "restart.sh", 0o755);
    let deeper = nested.join("old");
    fs::create_dir(&deeper).unwrap();
    fs::set_permissions(&deeper, fs::Permissions::from_mode(0o755)).unwrap();
    file(&deeper, "restart.sh", 0o755);
    let hidden = directory.root().join(".cache");
    fs::create_dir(&hidden).unwrap();
    fs::set_permissions(&hidden, fs::Permissions::from_mode(0o755)).unwrap();
    file(&hidden, "tool.sh", 0o755);
    file(directory.root(), ".secret.sh", 0o755);
    use crate::types::RefusalCode;
    assert_eq!(
        directory.resolve("media/old/restart.sh").unwrap_err().code,
        RefusalCode::TooDeep
    );
    assert_eq!(
        directory.resolve(".cache/tool.sh").unwrap_err().code,
        RefusalCode::Hidden
    );
    assert_eq!(
        directory.resolve(".secret.sh").unwrap_err().code,
        RefusalCode::Hidden
    );
    let names: Vec<String> = directory
        .list()
        .unwrap()
        .into_iter()
        .map(|entry| entry.path)
        .collect();
    assert_eq!(names, vec!["media/restart.sh"]);
}

#[test]
fn a_problem_names_its_code_and_the_path_it_concerns() {
    let (_folder, directory) = directory();
    file(directory.root(), "open.sh", 0o777);
    file(directory.root(), "notes.sh", 0o644);
    use crate::types::RefusalCode;
    let open = directory.resolve("open.sh").unwrap_err();
    assert_eq!(open.code, RefusalCode::Writable);
    assert!(open.path.ends_with("scripts/open.sh"));
    assert_eq!(
        directory.resolve("notes.sh").unwrap_err().code,
        RefusalCode::NotExecutable
    );
    fs::set_permissions(directory.root(), fs::Permissions::from_mode(0o775)).unwrap();
    file(directory.root(), "fine.sh", 0o755);
    let folder = directory.resolve("fine.sh").unwrap_err();
    assert_eq!(folder.code, RefusalCode::FolderWritable);
    assert!(folder.path.ends_with("scripts"));
}

#[test]
fn a_link_into_a_hidden_or_deep_path_is_refused() {
    let (_folder, directory) = directory();
    let hidden = directory.root().join(".cache");
    fs::create_dir(&hidden).unwrap();
    fs::set_permissions(&hidden, fs::Permissions::from_mode(0o755)).unwrap();
    file(&hidden, "tool.sh", 0o755);
    std::os::unix::fs::symlink(hidden.join("tool.sh"), directory.root().join("tool.sh")).unwrap();
    assert_eq!(
        directory.resolve("tool.sh").unwrap_err().code,
        crate::types::RefusalCode::Hidden
    );
}
