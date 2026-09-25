use std::time::Duration;

use serde_json::json;

use super::CaddyAdmin;
use crate::fakes::Caddy;
use crate::types::AdminAddress;

async fn admin() -> (Caddy, CaddyAdmin) {
    let (caddy, url) = Caddy::on_tcp().await;
    let admin = CaddyAdmin::new(&AdminAddress::parse(&url).unwrap()).unwrap();
    (caddy, admin)
}

#[tokio::test]
async fn a_configuration_is_loaded_as_json_and_read_back() {
    let (caddy, admin) = admin().await;
    let configuration = json!({ "apps": { "http": {} } });
    admin.load(&configuration).await.unwrap();
    assert_eq!(caddy.with(|recorded| recorded.loads), 1);
    assert_eq!(
        caddy.with(|recorded| recorded.content_types.clone()),
        vec!["application/json"]
    );
    assert_eq!(admin.config().await.unwrap(), configuration);
}

#[tokio::test]
async fn a_refusal_carries_caddys_own_message() {
    let (caddy, admin) = admin().await;
    caddy.with(|recorded| recorded.refuse = Some("loading config: unknown module".into()));
    let error = admin.load(&json!({})).await.unwrap_err();
    assert!(error.answered());
    let error = error.to_string();
    assert!(error.contains("400"), "{error}");
    assert!(error.contains("loading config: unknown module"), "{error}");
}

#[tokio::test]
async fn the_root_certificate_is_taken_from_the_local_authority() {
    let (_caddy, admin) = admin().await;
    assert_eq!(admin.root_certificate().await.unwrap(), Caddy::ROOT);
}

#[tokio::test]
async fn nothing_listening_is_reported_as_unreachable() {
    let admin = CaddyAdmin::new(&AdminAddress::parse("http://127.0.0.1:9").unwrap()).unwrap();
    let error = admin.config().await.unwrap_err();
    assert!(!error.answered());
    let error = error.to_string();
    assert!(error.contains("cannot be reached"), "{error}");
}

#[tokio::test(start_paused = true)]
async fn every_call_gives_up_after_five_seconds() {
    let (caddy, admin) = admin().await;
    caddy.with(|recorded| recorded.delay = Some(Duration::from_secs(30)));
    let error = admin.config().await.unwrap_err().to_string();
    assert!(error.contains("within 5 seconds"), "{error}");
}

#[cfg(unix)]
#[tokio::test]
async fn a_unix_socket_is_spoken_to_like_a_port() {
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("admin.sock");
    let caddy = Caddy::on_socket(&socket).await;
    let address = AdminAddress::parse(&format!("unix:{}", socket.display())).unwrap();
    let admin = CaddyAdmin::new(&address).unwrap();
    admin.load(&json!({ "admin": {} })).await.unwrap();
    assert_eq!(caddy.with(|recorded| recorded.loads), 1);
}

#[tokio::test]
async fn a_release_is_found_by_its_tag_and_a_missing_one_names_its_url() {
    let releases = crate::fakes::Releases::serve(
        crate::fakes::Releases::archive_of(crate::fakes::SCRIPT),
        false,
        None,
    )
    .await;
    let client = super::Releases::new().unwrap();
    let tag = format!("{}/tags/v{}", releases.base, crate::fakes::VERSION);
    assert_eq!(
        client.release(&tag).await.unwrap().version(),
        crate::fakes::VERSION
    );
    let missing = format!("{}/tags/v1.2.3", releases.base);
    let error = client.release(&missing).await.unwrap_err();
    assert!(error.contains(&missing), "{error}");
}
