use std::fs;

use super::atomic_write::PREVIOUS_SUFFIX;
use super::{resolve_configuration_path, write_atomically};

#[test]
fn without_the_variable_the_file_is_home_portal_toml_in_the_working_directory() {
    assert_eq!(
        resolve_configuration_path(None).to_str(),
        Some("home-portal.toml")
    );
    assert_eq!(
        resolve_configuration_path(Some("/etc/home-portal/portal.toml".into())).to_str(),
        Some("/etc/home-portal/portal.toml")
    );
}

#[test]
fn an_atomic_write_replaces_the_file_and_keeps_the_previous_content() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, "old = 1\n").unwrap();
    write_atomically(&path, b"old = 1\n", "new = 2\n").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "new = 2\n");
    let previous = directory
        .path()
        .join(format!("home-portal.toml{PREVIOUS_SUFFIX}"));
    assert_eq!(fs::read_to_string(previous).unwrap(), "old = 1\n");
}

#[cfg(unix)]
#[test]
fn a_written_file_is_readable_by_its_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, "a = 1\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    write_atomically(&path, b"a = 1\n", "a = 2\n").unwrap();
    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}
