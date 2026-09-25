use std::fs;
use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use portal_config::ConfigStore;
use portal_feature::{EventName, Feature, PortalEvent, Principal};
use tempfile::TempDir;

use super::catalogue::{FILE, revision, send, service_json, write};
use crate::ServicesFeature;
use crate::fakes::{Recorder, Switch};
use crate::types::ServicesPorts;

fn announcing() -> (Router, Arc<Recorder>, TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, FILE).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let events = Arc::new(Recorder::default());
    let feature = ServicesFeature::new(
        store.clone(),
        portal_model::Environment::internet(),
        ServicesPorts {
            observers: Vec::new(),
            publishing: Arc::new(Switch::on()),
            events: events.clone(),
        },
    )
    .unwrap();
    store.adopt(vec![feature.validator().unwrap()]).unwrap();
    let router = feature.router().layer(axum::Extension(Principal {
        name: "admin".into(),
    }));
    (router, events, directory)
}

fn recorded(events: &Recorder) -> Vec<PortalEvent> {
    events.events.lock().unwrap().clone()
}

#[tokio::test]
async fn creating_renaming_and_deleting_a_service_are_each_announced_once() {
    let (router, events, _directory) = announcing();
    let current = revision(&router).await;
    let (status, _, _) = send(
        &router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&current),
            &service_json("nas", "http://10.255.0.9"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let current = revision(&router).await;
    let (status, _, _) = send(
        &router,
        write(
            "PUT",
            "/api/services/nas",
            Some(&current),
            &service_json("storage", "http://10.255.0.9"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let current = revision(&router).await;
    let (status, _, _) = send(
        &router,
        write("DELETE", "/api/services/storage", Some(&current), ""),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let found = recorded(&events);
    let names: Vec<EventName> = found.iter().map(|event| event.name).collect();
    assert_eq!(
        names,
        vec![
            EventName::ServiceCreated,
            EventName::ServiceUpdated,
            EventName::ServiceDeleted
        ]
    );
    assert_eq!(found[0].value("service.id"), Some("nas"));
    assert_eq!(found[0].value("user.name"), Some("admin"));
    assert_eq!(found[1].value("service.id"), Some("storage"));
    assert_eq!(found[1].value("service.previous_id"), Some("nas"));
    assert_eq!(found[2].value("service.id"), Some("storage"));
    assert_eq!(found[2].value("service.name"), Some("New"));
}

#[tokio::test]
async fn a_refused_write_announces_nothing() {
    let (router, events, _directory) = announcing();
    let (status, _, _) = send(
        &router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some("\"stale\""),
            &service_json("nas", "http://10.255.0.9"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let current = revision(&router).await;
    let (status, _, _) = send(
        &router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&current),
            &service_json("nas", "ftp://nowhere"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(recorded(&events).is_empty());
}
