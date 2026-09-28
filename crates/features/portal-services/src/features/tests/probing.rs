use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use portal_config::ConfigStore;
use portal_model::ServiceState;

use crate::ServicesFeature;
use crate::fakes::{Recorder, Switch};
use crate::types::ServicesPorts;

fn answering() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok");
        }
    });
    port
}

fn feature_for(port: u16) -> (tempfile::TempDir, ServicesFeature) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        format!("[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://127.0.0.1:{port}\"\n"),
    )
    .unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = ServicesFeature::new(
        store,
        portal_model::Environment::internet(),
        ServicesPorts {
            observers: Vec::new(),
            publishing: Arc::new(Switch::on()),
            events: Arc::new(Recorder::default()),
        },
    )
    .unwrap();
    (directory, feature)
}

#[tokio::test]
async fn probing_from_a_workflow_answers_and_updates_the_status() {
    let (_directory, feature) = feature_for(answering());
    assert_eq!(feature.status_of("nas").state, ServiceState::Unknown);
    let outcome = feature.probe_service().run("nas").await.unwrap();
    assert_eq!(outcome.state, ServiceState::Up);
    assert_eq!(
        feature.current_status().run("nas").unwrap().state,
        ServiceState::Up
    );
}

#[tokio::test]
async fn an_unknown_service_is_refused_by_both() {
    let (_directory, feature) = feature_for(answering());
    assert_eq!(
        feature.probe_service().run("ghost").await.unwrap_err(),
        "no service ghost"
    );
    assert_eq!(
        feature.current_status().run("ghost").unwrap_err(),
        "no service ghost"
    );
}
