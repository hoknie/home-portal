use std::fs;

use super::atomic_write::PREVIOUS_SUFFIX;
use super::{resolve_configuration_path, stray_configuration, write_atomically};
use crate::types::ConfigError;

fn path_of(
    configuration: Option<&str>,
    xdg: Option<&str>,
    home: Option<&str>,
) -> Result<(String, bool), String> {
    resolve_configuration_path(
        configuration.map(Into::into),
        xdg.map(Into::into),
        home.map(Into::into),
    )
    .map(|location| (location.path.display().to_string(), location.by_default))
    .map_err(|error| error.to_string())
}

#[test]
fn the_default_path_is_in_the_home_directory() {
    assert_eq!(
        path_of(None, None, Some("/home/anna")),
        Ok((
            "/home/anna/.config/home-portal/home-portal.toml".into(),
            true
        ))
    );
}

#[test]
fn an_absolute_xdg_config_home_is_honoured_and_a_relative_one_ignored() {
    assert_eq!(
        path_of(None, Some("/srv/settings"), Some("/home/anna")),
        Ok(("/srv/settings/home-portal/home-portal.toml".into(), true))
    );
    assert_eq!(
        path_of(None, Some("settings"), Some("/home/anna")),
        Ok((
            "/home/anna/.config/home-portal/home-portal.toml".into(),
            true
        ))
    );
}

#[test]
fn the_variable_still_wins_and_an_empty_one_counts_as_unset() {
    assert_eq!(
        path_of(
            Some("/etc/home-portal/portal.toml"),
            Some("/srv"),
            Some("/home/anna")
        ),
        Ok(("/etc/home-portal/portal.toml".into(), false))
    );
    assert_eq!(
        path_of(Some(""), None, Some("/home/anna")),
        Ok((
            "/home/anna/.config/home-portal/home-portal.toml".into(),
            true
        ))
    );
}

#[test]
fn without_home_the_error_names_both_variables() {
    let message = path_of(None, None, Some("")).unwrap_err();
    assert!(message.contains("HOME_PORTAL_CONFIG"), "{message}");
    assert!(message.contains("HOME so that"), "{message}");
    assert!(path_of(None, Some("relative"), None).is_err());
}

#[test]
fn a_file_left_in_the_working_directory_is_named_in_the_missing_message() {
    let working = tempfile::tempdir().unwrap();
    assert_eq!(stray_configuration(working.path()), None);
    fs::write(working.path().join("home-portal.toml"), "").unwrap();
    let stray = stray_configuration(working.path()).unwrap();
    let message = ConfigError::Missing {
        path: "/home/anna/.config/home-portal/home-portal.toml".into(),
        stray: Some(stray.clone()),
    }
    .to_string();
    assert!(
        message.contains("/home/anna/.config/home-portal/home-portal.toml"),
        "{message}"
    );
    assert!(message.contains(&stray.display().to_string()), "{message}");
    assert!(message.contains("HOME_PORTAL_CONFIG"), "{message}");
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
