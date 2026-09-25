use std::fs;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use tower::ServiceExt;

use super::SecretsFeature;

#[tokio::test]
async fn the_list_names_every_secret_and_whether_it_is_set_but_never_its_value() {
    let directory = tempfile::tempdir().unwrap();
    let main = directory.path().join("home-portal.toml");
    let secrets = directory.path().join("secrets.toml");
    fs::write(&main, "include = [\"secrets.toml\"]\n\n[notifications.telegram]\nsecret = \"telegram_token\"\n\n[[dashboard.widgets]]\ntype = \"calendar\"\nsettings = { secret = \"calendar_password\" }\n").unwrap();
    fs::write(&secrets, "[secrets]\ntelegram_token = \"hunter2\"\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&secrets, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let feature = SecretsFeature::new(Arc::new(ConfigStore::open(&main).unwrap()));
    let response = feature
        .router()
        .oneshot(
            Request::get(SecretsFeature::PATH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert_eq!(
        body,
        r#"{"secrets":[{"name":"calendar_password","set":false},{"name":"telegram_token","set":true}]}"#
    );
    assert!(!body.contains("hunter2"));
}
