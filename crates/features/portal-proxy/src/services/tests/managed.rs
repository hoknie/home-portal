use std::fs;
use std::sync::Arc;
use std::time::Duration;

use crate::clients::Releases as Client;
use crate::fakes::{Releases, SCRIPT, VERSION};
use crate::services::CaddyManager;
use crate::services::caddy_installer::install;
use crate::services::caddy_launcher::{launch, log_tail};
use crate::types::{AdminAddress, CaddyHome, CaddySource, CaddyVersion, DownloadStage, Platform};

fn home() -> (tempfile::TempDir, CaddyHome) {
    let directory = tempfile::tempdir().unwrap();
    let home = CaddyHome::beside(&directory.path().join("home-portal.toml"));
    (directory, home)
}

fn source(releases: &Releases, version: &str) -> CaddySource {
    CaddySource {
        base: releases.base.clone(),
        version: CaddyVersion::parse(version).unwrap(),
    }
}

async fn installed_at(home: &CaddyHome, tamper: bool, version: &str) -> Result<String, String> {
    let releases = Releases::serve(Releases::archive_of(SCRIPT), tamper, None).await;
    install(
        home,
        &Client::new().unwrap(),
        &Platform::current().unwrap(),
        &source(&releases, version),
    )
    .await
}

async fn installed_in(home: &CaddyHome, tamper: bool) -> Result<String, String> {
    installed_at(home, tamper, CaddyVersion::LATEST).await
}

#[tokio::test]
async fn a_verified_release_is_installed_executable_with_its_version() {
    let (_directory, home) = home();
    assert_eq!(installed_in(&home, false).await.unwrap(), VERSION);
    assert_eq!(home.installed().as_deref(), Some(VERSION));
    assert_eq!(fs::read_to_string(home.binary()).unwrap(), SCRIPT);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(home.binary()).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
    }
    assert!(!home.staging().exists());
    let origin = home.installed_from().unwrap();
    assert!(origin.starts_with("http://127.0.0.1:"), "{origin}");
    assert!(origin.ends_with("/archive"), "{origin}");
}

#[tokio::test]
async fn a_pinned_version_is_fetched_by_its_tag() {
    let (_directory, home) = home();
    assert_eq!(installed_at(&home, false, VERSION).await.unwrap(), VERSION);
    assert_eq!(home.installed().as_deref(), Some(VERSION));
}

#[tokio::test]
async fn a_version_that_does_not_exist_installs_nothing_and_names_the_url() {
    let (_directory, home) = home();
    let error = installed_at(&home, false, "1.2.3").await.unwrap_err();
    assert!(error.contains("/tags/v1.2.3"), "{error}");
    assert!(home.installed().is_none());
}

#[test]
fn a_binary_without_a_recorded_origin_has_none() {
    let (_directory, home) = home();
    fs::create_dir_all(&home.directory).unwrap();
    fs::write(home.binary(), SCRIPT).unwrap();
    fs::write(home.version_file(), VERSION).unwrap();
    assert_eq!(home.installed().as_deref(), Some(VERSION));
    assert_eq!(home.installed_from(), None);
}

#[tokio::test]
async fn an_archive_that_does_not_match_its_checksum_installs_nothing() {
    let (_directory, home) = home();
    let error = installed_in(&home, true).await.unwrap_err();
    assert!(error.contains("SHA-512"), "{error}");
    assert!(home.installed().is_none());
}

#[test]
fn an_unsupported_system_is_named() {
    let error = Platform::of("windows", "x86_64").unwrap_err();
    assert!(error.contains("windows"), "{error}");
    assert_eq!(
        Platform::of("linux", "aarch64").unwrap().archive("2.11.4"),
        "caddy_2.11.4_linux_arm64.tar.gz"
    );
    assert_eq!(
        Platform::of("macos", "x86_64").unwrap().archive("2.11.4"),
        "caddy_2.11.4_mac_amd64.tar.gz"
    );
}

#[tokio::test]
async fn a_second_download_while_one_runs_is_refused_and_the_first_finishes() {
    let (_directory, home) = home();
    let releases = Releases::serve(
        Releases::archive_of(SCRIPT),
        false,
        Some(Duration::from_millis(300)),
    )
    .await;
    let manager = Arc::new(CaddyManager::new(home.clone()));
    let source = source(&releases, CaddyVersion::LATEST);
    manager.begin_download(source.clone()).unwrap();
    assert_eq!(manager.download_state().state, DownloadStage::Downloading);
    assert_eq!(manager.begin_download(source), Err(CaddyManager::BUSY));
    for _ in 0..100 {
        if manager.download_state().state != DownloadStage::Downloading {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(manager.download_state().state, DownloadStage::Installed);
    assert_eq!(manager.installed().as_deref(), Some(VERSION));
}

#[cfg(unix)]
#[tokio::test]
async fn caddy_runs_resumed_in_its_own_process_group_with_its_data_under_caddy() {
    let (_directory, home) = home();
    installed_in(&home, false).await.unwrap();
    let admin = AdminAddress::parse("http://127.0.0.1:12345").unwrap();
    launch(&home, &admin).unwrap();
    let record = home.directory.join("launches.txt");
    for _ in 0..100 {
        if record.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let line = fs::read_to_string(&record).unwrap();
    let parts: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(
        parts[0], parts[1],
        "the process leads its own group: {line}"
    );
    assert_eq!(parts[2], home.data().display().to_string());
    assert_eq!(
        &parts[3..],
        [
            "run",
            "--resume",
            "--config",
            &home.initial().display().to_string()
        ]
    );
    let initial = fs::read_to_string(home.initial()).unwrap();
    assert_eq!(initial, r#"{"admin":{"listen":"127.0.0.1:12345"}}"#);
    let _ = std::process::Command::new("kill").arg(parts[0]).status();
}

#[test]
fn the_log_tail_keeps_the_last_lines() {
    let (_directory, home) = home();
    fs::create_dir_all(&home.directory).unwrap();
    let text: String = (1..=30).map(|line| format!("line {line}\n")).collect();
    fs::write(home.log(), text).unwrap();
    let tail = log_tail(&home, 20);
    assert_eq!(tail.len(), 20);
    assert_eq!(tail[0], "line 11");
    assert_eq!(tail[19], "line 30");
}

#[test]
fn the_caddy_directory_is_absolute_even_for_a_relative_configuration_path() {
    let home = crate::types::CaddyHome::beside(std::path::Path::new("home-portal.toml"));
    assert!(home.directory.is_absolute(), "{}", home.directory.display());
    assert_eq!(
        home.directory,
        std::env::current_dir().unwrap().join("caddy")
    );
    assert!(home.binary().is_absolute());
}
