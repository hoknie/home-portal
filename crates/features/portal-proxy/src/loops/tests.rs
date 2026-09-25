use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use portal_config::ConfigStore;
use serde_json::json;
use tempfile::TempDir;

use super::CaddySync;
use crate::fakes::{Caddy, Catalogue};
use crate::services::CaddyManager;
use crate::types::{CaddyHome, Cadence, SyncSetup};

const FAST: Cadence = Cadence {
    tick: Duration::from_millis(20),
    check_every: Duration::from_millis(300),
    retry_every: Duration::from_millis(100),
    restart_every: Duration::from_millis(200),
};

struct Setup {
    sync: Arc<CaddySync>,
    path: PathBuf,
    _directory: TempDir,
}

fn file(admin: &str, services: &str) -> String {
    format!(
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nenabled = true\nadmin = \"{admin}\"\nportal_host = \"portal.example.com\"\n{services}"
    )
}

fn setup(text: &str) -> Setup {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let configuration = Arc::new(ConfigStore::open(&path).unwrap());
    let catalogue = Arc::new(Catalogue {
        configuration: configuration.clone(),
    });
    let manager = Arc::new(CaddyManager::new(CaddyHome::at(
        path.with_file_name("caddy"),
    )));
    let sync = Arc::new(CaddySync::new(
        configuration,
        catalogue,
        SyncSetup {
            portal: "127.0.0.1:8080".parse().unwrap(),
            cadence: FAST,
            manager,
        },
    ));
    Setup {
        sync,
        path,
        _directory: directory,
    }
}

async fn eventually(condition: impl Fn() -> bool) -> bool {
    for _ in 0..100 {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

#[tokio::test]
async fn caddy_down_at_start_is_recorded_without_stopping_anything() {
    let setup = setup(&file("http://127.0.0.1:9", ""));
    let problem = setup.sync.apply().await.unwrap_err();
    assert!(!problem.answered());
    let state = setup.sync.state();
    assert!(!state.reachable);
    assert!(state.last_error.unwrap().contains("cannot be reached"));
    assert!(state.applied.is_none());
}

#[tokio::test]
async fn the_loop_loads_at_start_and_again_after_the_file_changes() {
    let (caddy, url) = Caddy::on_tcp().await;
    let setup = setup(&file(&url, ""));
    tokio::spawn(setup.sync.clone().run());
    assert!(eventually(|| caddy.with(|recorded| recorded.loads) == 1).await);
    let state = setup.sync.state();
    assert!(state.reachable);
    assert!(state.last_applied_at.is_some());
    let services = "\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://192.168.1.10:8096\"\nproxy = { host = \"media.example.com\" }\n";
    tokio::time::sleep(Duration::from_millis(1100)).await;
    fs::write(&setup.path, file(&url, services)).unwrap();
    assert!(
        eventually(|| caddy.with(|recorded| recorded
            .configuration
            .to_string()
            .contains("media.example.com")))
        .await
    );
    assert_eq!(caddy.with(|recorded| recorded.loads), 2);
}

#[tokio::test]
async fn a_configuration_changed_behind_the_portals_back_is_replaced_on_the_next_check() {
    let (caddy, url) = Caddy::on_tcp().await;
    let setup = setup(&file(&url, ""));
    tokio::spawn(setup.sync.clone().run());
    assert!(eventually(|| caddy.with(|recorded| recorded.loads) == 1).await);
    caddy.with(|recorded| {
        recorded.configuration = json!({ "apps": { "http": { "servers": { "mine": {} } } } })
    });
    assert!(eventually(|| caddy.with(|recorded| recorded.loads) == 2).await);
    let rendered = setup.sync.rendered().unwrap().1;
    assert_eq!(
        caddy.with(|recorded| recorded.configuration.clone()),
        rendered
    );
}

#[tokio::test]
async fn an_equal_configuration_is_not_loaded_again() {
    let (caddy, url) = Caddy::on_tcp().await;
    let setup = setup(&file(&url, ""));
    tokio::spawn(setup.sync.clone().run());
    assert!(eventually(|| caddy.with(|recorded| recorded.loads) == 1).await);
    let applied_at = setup.sync.state().last_applied_at;
    tokio::time::sleep(FAST.check_every * 3).await;
    assert_eq!(caddy.with(|recorded| recorded.loads), 1);
    assert_eq!(setup.sync.state().last_applied_at, applied_at);
}

#[tokio::test]
async fn a_refusal_counts_as_an_answer() {
    let (caddy, url) = Caddy::on_tcp().await;
    caddy.with(|recorded| recorded.refuse = Some("unknown module".into()));
    let setup = setup(&file(&url, ""));
    assert!(setup.sync.apply().await.is_err());
    let state = setup.sync.state();
    assert!(state.reachable);
    assert!(state.last_error.unwrap().contains("unknown module"));
}

#[tokio::test]
async fn a_disabled_proxy_is_never_contacted() {
    let (caddy, url) = Caddy::on_tcp().await;
    let setup = setup(&file(&url, "").replace("enabled = true", "enabled = false"));
    tokio::spawn(setup.sync.clone().run());
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(caddy.with(|recorded| recorded.loads), 0);
    assert!(setup.sync.rendered().is_none());
}

#[cfg(unix)]
fn install_fake_caddy(path: &std::path::Path) -> CaddyHome {
    use std::os::unix::fs::PermissionsExt;
    let home = CaddyHome::at(path.with_file_name("caddy"));
    fs::create_dir_all(&home.directory).unwrap();
    fs::write(home.binary(), crate::fakes::SCRIPT).unwrap();
    fs::set_permissions(home.binary(), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(home.version_file(), crate::fakes::VERSION).unwrap();
    home
}

#[cfg(unix)]
fn launches(home: &CaddyHome) -> Vec<String> {
    fs::read_to_string(home.directory.join("launches.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

#[cfg(unix)]
fn stop_fakes(home: &CaddyHome) {
    for line in launches(home) {
        if let Some(pid) = line.split_whitespace().next() {
            let _ = std::process::Command::new("kill").arg(pid).status();
        }
    }
}

#[cfg(unix)]
#[tokio::test]
async fn a_managed_caddy_that_does_not_answer_is_started_again() {
    let setup = setup(
        &file("http://127.0.0.1:9", "").replace("enabled = true", "enabled = true\nmanaged = true"),
    );
    let home = install_fake_caddy(&setup.path);
    tokio::spawn(setup.sync.clone().run());
    assert!(
        eventually(|| launches(&home).len() >= 2).await,
        "{:?}",
        launches(&home)
    );
    stop_fakes(&home);
}

#[cfg(unix)]
#[tokio::test]
async fn an_unmanaged_caddy_is_never_started() {
    let setup = setup(&file("http://127.0.0.1:9", ""));
    let home = install_fake_caddy(&setup.path);
    tokio::spawn(setup.sync.clone().run());
    tokio::time::sleep(FAST.restart_every * 3).await;
    assert!(launches(&home).is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn a_managed_caddy_that_answers_is_left_alone() {
    let (_caddy, url) = Caddy::on_tcp().await;
    let setup = setup(&file(&url, "").replace("enabled = true", "enabled = true\nmanaged = true"));
    let home = install_fake_caddy(&setup.path);
    tokio::spawn(setup.sync.clone().run());
    tokio::time::sleep(FAST.restart_every * 3).await;
    assert!(launches(&home).is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn a_process_holding_the_admin_address_without_answering_is_not_joined_by_another() {
    let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", silent.local_addr().unwrap());
    let setup =
        setup(&file(&address, "").replace("enabled = true", "enabled = true\nmanaged = true"));
    let home = install_fake_caddy(&setup.path);
    tokio::spawn(setup.sync.clone().run());
    tokio::time::sleep(FAST.restart_every * 3).await;
    assert!(launches(&home).is_empty());
    for _ in 0..80 {
        if setup.sync.state().last_error.as_deref() == Some(super::caddy_sync::HELD) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        setup.sync.state().last_error.as_deref(),
        Some(super::caddy_sync::HELD)
    );
    assert!(launches(&home).is_empty());
    drop(silent);
}
