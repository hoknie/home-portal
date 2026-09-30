use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;

use super::address::ADDRESS_VARIABLE;
use super::interface::interface_folder;
use super::shutdown::requested;
use super::start::replaced_executable;
use super::{parse_address, resolve_address};
use crate::types::{BootError, Ended, Restart, Signals};

fn store(text: &str) -> (tempfile::TempDir, Arc<ConfigStore>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
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

#[test]
fn a_replaced_executable_is_found_at_its_path() {
    let directory = tempfile::tempdir().unwrap();
    let binary = directory.path().join("home-portal");
    fs::write(&binary, "").unwrap();
    let deleted = directory.path().join("home-portal (deleted)");
    assert_eq!(replaced_executable(deleted), binary);
    let gone = directory.path().join("gone (deleted)");
    assert_eq!(replaced_executable(gone.clone()), gone);
    assert_eq!(replaced_executable(binary.clone()), binary);
}

#[tokio::test]
async fn a_restart_request_ends_the_serving_with_a_restart_and_a_second_is_harmless() {
    let restart = Restart::default();
    assert!(!restart.requested());
    let waiting = tokio::spawn(requested(Signals::install(), restart.clone()));
    restart.request();
    restart.request();
    let ended = tokio::time::timeout(std::time::Duration::from_secs(2), waiting)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ended, Ended::Restart);
    assert!(restart.requested());
    assert_eq!(requested(Signals::install(), restart).await, Ended::Restart);
}

#[test]
fn the_interface_is_looked_for_in_the_variable_then_beside_the_binary_then_in_share() {
    let prefix = tempfile::tempdir().unwrap();
    let bin = prefix.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let binary = bin.join("home-portal");
    fs::write(&binary, "").unwrap();
    let link_folder = tempfile::tempdir().unwrap();
    let link = link_folder.path().join("home-portal");
    std::os::unix::fs::symlink(&binary, &link).unwrap();
    let real = fs::canonicalize(prefix.path()).unwrap();
    assert_eq!(
        interface_folder(None, Some(link)),
        vec![real.join("bin/web"), real.join("share/home-portal/web")]
    );
    assert_eq!(
        interface_folder(Some("/srv/web".into()), Some(binary.clone())),
        vec![std::path::PathBuf::from("/srv/web")]
    );
    assert_eq!(interface_folder(Some("".into()), Some(binary)).len(), 2);
    assert!(interface_folder(None, None).is_empty());
}

async fn listening(
    router: axum::Router,
    limits: super::serve::Limits,
) -> (
    std::net::SocketAddr,
    tokio::sync::oneshot::Sender<()>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let serving = tokio::spawn(async move {
        super::serve::accept_until(
            listener,
            router,
            async move {
                let _ = stopped.await;
                Ended::Stopped
            },
            limits,
        )
        .await
        .unwrap();
    });
    (address, stop, serving)
}

fn short() -> super::serve::Limits {
    super::serve::Limits {
        header_read: std::time::Duration::from_millis(300),
        drain: std::time::Duration::from_millis(500),
    }
}

#[tokio::test]
async fn a_client_that_never_finishes_its_headers_is_dropped() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (address, _stop, _serving) = listening(axum::Router::new(), short()).await;
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nhost: x\r\n")
        .await
        .unwrap();
    let mut answer = Vec::new();
    let closed = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.read_to_end(&mut answer),
    )
    .await;
    assert!(closed.is_ok(), "the connection stayed open");
}

#[tokio::test]
async fn a_stuck_request_does_not_hold_the_shutdown_past_its_deadline() {
    use tokio::io::AsyncWriteExt;
    let router = axum::Router::new().route(
        "/hang",
        axum::routing::get(|| async {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            "never"
        }),
    );
    let (address, stop, serving) = listening(router, short()).await;
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET /hang HTTP/1.1\r\nhost: x\r\n\r\n")
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    stop.send(()).unwrap();
    let finished = tokio::time::timeout(std::time::Duration::from_secs(3), serving).await;
    assert!(finished.is_ok(), "shutdown waited for the stuck request");
}
