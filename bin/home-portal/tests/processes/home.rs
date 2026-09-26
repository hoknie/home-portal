use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use portal_auth::hash_password;

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");

#[test]
fn a_file_left_in_the_working_directory_is_named_when_the_default_is_missing() {
    let home = tempfile::tempdir().unwrap();
    let working = tempfile::tempdir().unwrap();
    fs::write(working.path().join("home-portal.toml"), "").unwrap();
    let output = Command::new(BINARY)
        .current_dir(working.path())
        .env_remove("HOME_PORTAL_CONFIG")
        .env_remove("XDG_CONFIG_HOME")
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let message = String::from_utf8_lossy(&output.stderr);
    let expected = home.path().join(".config/home-portal/home-portal.toml");
    assert!(
        message.contains(&expected.display().to_string()),
        "{message}"
    );
    assert!(
        message.contains("found in the working directory"),
        "{message}"
    );
    assert!(message.contains("HOME_PORTAL_CONFIG"), "{message}");
}

#[cfg(unix)]
#[test]
fn without_the_variable_the_portal_reads_and_writes_in_the_home_configuration_folder() {
    let home = tempfile::tempdir().unwrap();
    let folder = home.path().join(".config/home-portal");
    fs::create_dir_all(&folder).unwrap();
    let hash = hash_password("secret").unwrap();
    fs::write(
        folder.join("home-portal.toml"),
        format!("[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n"),
    )
    .unwrap();
    let working = tempfile::tempdir().unwrap();
    let mut child = Command::new(BINARY)
        .current_dir(working.path())
        .env_remove("HOME_PORTAL_CONFIG")
        .env_remove("XDG_CONFIG_HOME")
        .env("HOME", home.path())
        .env("HOME_PORTAL_ADDRESS", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let began = Instant::now();
    while !folder.join("icons").is_dir() {
        assert!(child.try_wait().unwrap().is_none(), "the portal exited");
        assert!(
            began.elapsed() < Duration::from_secs(20),
            "the portal did not start"
        );
        thread::sleep(Duration::from_millis(50));
    }
    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    assert!(child.wait().unwrap().success());
    assert!(folder.join("icons").is_dir());
    assert!(!working.path().join("icons").exists());
}
