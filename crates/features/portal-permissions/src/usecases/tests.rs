use std::fs;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use portal_config::ConfigStore;
use portal_feature::ApiError;
use tempfile::TempDir;

use crate::fakes::FixedChecks;
use crate::features::PermissionsFeature;
use crate::types::{Limits, Owner, PermissionCode, PermissionState, PermissionsView};

struct Portal {
    feature: PermissionsFeature,
    _folder: TempDir,
}

fn limits(prompt: Duration) -> Limits {
    Limits {
        prompt,
        ..Limits::default()
    }
}

fn portal(configuration: &str, source: FixedChecks, prompt: Duration) -> Portal {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, configuration).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    Portal {
        feature: PermissionsFeature::with(
            store,
            Arc::new(source),
            Owner::detect(None, None),
            limits(prompt),
        ),
        _folder: folder,
    }
}

fn state_of(view: &PermissionsView, code: &str) -> PermissionState {
    view.permissions
        .iter()
        .find(|permission| permission.code.code() == code)
        .unwrap()
        .state
}

#[tokio::test]
async fn before_any_request_every_state_is_unknown() {
    let portal = portal(
        "",
        FixedChecks::granting(Duration::ZERO),
        Duration::from_secs(5),
    );
    let view = portal.feature.show_permissions().run();
    assert_eq!(view.permissions.len(), 7);
    assert!(
        view.permissions
            .iter()
            .all(|permission| permission.state == PermissionState::Unknown
                && permission.learned_at.is_none())
    );
}

#[tokio::test]
async fn a_request_records_each_answer_and_a_denial_is_reported() {
    let mut source = FixedChecks::granting(Duration::ZERO);
    source.denied = vec![PermissionCode::RemovableVolumes];
    let portal = portal("", source, Duration::from_secs(5));
    portal.feature.request_permissions().run().await.unwrap();
    let view = portal.feature.show_permissions().run();
    assert_eq!(state_of(&view, "local-network"), PermissionState::Granted);
    assert_eq!(
        state_of(&view, "removable-volumes"),
        PermissionState::Denied
    );
    assert!(view.denied());
    assert!(
        view.permissions
            .iter()
            .all(|permission| permission.learned_at.is_some())
    );
}

#[tokio::test]
async fn a_second_request_while_one_runs_is_refused() {
    let portal = portal(
        "",
        FixedChecks::granting(Duration::from_millis(300)),
        Duration::from_secs(5),
    );
    let request = portal.feature.request_permissions();
    request.start().unwrap();
    assert!(matches!(request.start(), Err(ApiError::Conflict(_))));
    assert!(matches!(request.run().await, Err(ApiError::Conflict(_))));
}

#[tokio::test]
async fn an_answer_slower_than_the_limit_stays_pending_and_is_not_asked_twice() {
    let portal = portal(
        "",
        FixedChecks::granting(Duration::from_millis(400)),
        Duration::from_millis(50),
    );
    let request = portal.feature.request_permissions();
    request.run().await.unwrap();
    let view = portal.feature.show_permissions().run();
    assert_eq!(
        state_of(&view, "folder:Documents"),
        PermissionState::Pending
    );
    request.run().await.unwrap();
    assert_eq!(
        state_of(&portal.feature.show_permissions().run(), "folder:Documents"),
        PermissionState::Pending
    );
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(
        state_of(&portal.feature.show_permissions().run(), "folder:Documents"),
        PermissionState::Granted
    );
}

#[tokio::test]
async fn switched_off_at_start_only_full_disk_access_is_checked() {
    let portal = portal(
        "[permissions]\nrequest_at_start = false\n",
        FixedChecks::granting(Duration::ZERO),
        Duration::from_secs(5),
    );
    portal.feature.request_permissions().at_start().await;
    let view = portal.feature.show_permissions().run();
    assert_eq!(
        state_of(&view, "full-disk-access"),
        PermissionState::Granted
    );
    assert_eq!(state_of(&view, "local-network"), PermissionState::Unknown);
    assert_eq!(
        state_of(&view, "automation:System Events"),
        PermissionState::Unknown
    );
}

#[tokio::test]
async fn at_start_every_permission_is_asked_for_by_default() {
    let source = FixedChecks::granting(Duration::ZERO);
    let asked = source.asked.clone();
    let portal = portal("", source, Duration::from_secs(5));
    portal.feature.request_permissions().at_start().await;
    assert_eq!(asked.load(Ordering::SeqCst), 1);
    let view = portal.feature.show_permissions().run();
    assert!(
        view.permissions
            .iter()
            .all(|permission| permission.state == PermissionState::Granted)
    );
}

#[tokio::test]
async fn on_another_system_every_permission_does_not_apply() {
    let mut source = FixedChecks::granting(Duration::ZERO);
    source.applies = false;
    let portal = portal("", source, Duration::from_secs(5));
    let view = portal.feature.show_permissions().run();
    assert!(
        view.permissions
            .iter()
            .all(|permission| permission.state == PermissionState::NotApplicable)
    );
}
