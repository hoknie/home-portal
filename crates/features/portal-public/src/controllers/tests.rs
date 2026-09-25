use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_feature::{ApiError, Feature};
use portal_model::{DetectedEnvironment, Environment, ServiceStatus};
use portal_widget::{WidgetData, WidgetRegistry, WidgetSize};
use serde_json::{Value, json};
use time::macros::datetime;
use tower::ServiceExt;

use crate::features::PublicFeature;
use crate::ports::{PublicLayout, PublicServices};
use crate::responses::{PublicSection, PublicService, PublicWidget};

struct Catalogue;

impl PublicServices for Catalogue {
    fn public_services(&self, environment: &Environment) -> Vec<PublicService> {
        if environment.is_internet() {
            return vec![PublicService {
                id: "blog".into(),
                name: "Blog".into(),
                address: "https://blog.example.com".into(),
                group: None,
                icon: Some("/api/public/icons/blog".into()),
                description: None,
                status: None,
            }];
        }
        vec![PublicService {
            id: "nas".into(),
            name: "NAS".into(),
            address: "http://192.168.1.10".into(),
            group: Some("Home".into()),
            icon: None,
            description: None,
            status: Some(ServiceStatus::unknown(datetime!(2026-09-22 10:00 UTC))),
        }]
    }
}

struct Layout;

#[async_trait]
impl PublicLayout for Layout {
    async fn public_widgets(&self, environment: &Environment) -> Vec<PublicWidget> {
        if environment.is_internet() {
            return Vec::new();
        }
        vec![PublicWidget {
            kind: "weather".into(),
            id: Some("riga".into()),
            title: None,
            settings: json!({}),
            section: Some("now".into()),
            size: WidgetSize::Half,
        }]
    }

    async fn public_sections(&self, environment: &Environment) -> Vec<PublicSection> {
        if environment.is_internet() {
            return Vec::new();
        }
        vec![PublicSection {
            id: "now".into(),
            title: Some("Now".into()),
        }]
    }

    async fn public_data(
        &self,
        id: &str,
        environment: &Environment,
    ) -> Result<WidgetData, ApiError> {
        if environment.is_internet() || id != "riga" {
            return Err(ApiError::NotFound(WidgetRegistry::UNKNOWN_WIDGET));
        }
        Ok(WidgetData {
            data: json!({ "temperature": 12.0 }),
            fetched_at: datetime!(2026-09-22 10:00 UTC),
            stale: false,
            problem: None,
            refresh_seconds: 900,
        })
    }
}

async fn ask(path: &str, environment: Environment) -> (StatusCode, Value) {
    let feature = PublicFeature::new(Arc::new(Catalogue), Arc::new(Layout));
    let mut request = Request::get(path).body(Body::empty()).unwrap();
    let choices = if environment.is_internet() {
        Vec::new()
    } else {
        ["local", "vpn", "internet"]
            .into_iter()
            .map(|name| Environment::parse(name).unwrap())
            .collect()
    };
    request.extensions_mut().insert(DetectedEnvironment {
        environment: environment.clone(),
        choices,
    });
    request.extensions_mut().insert(environment);
    let response = feature.public_router().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn the_public_portal_answers_without_a_session_with_what_this_environment_may_see() {
    let (status, body) = ask(
        PublicFeature::PORTAL_PATH,
        Environment::parse("local").unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["environment"], "local");
    assert_eq!(body["services"][0]["id"], "nas");
    assert_eq!(body["services"][0]["address"], "http://192.168.1.10");
    assert_eq!(body["widgets"][0]["type"], "weather");
}

#[tokio::test]
async fn another_environment_sees_another_portal() {
    let (_, body) = ask(PublicFeature::PORTAL_PATH, Environment::internet()).await;
    assert_eq!(body["services"][0]["id"], "blog");
    assert!(body["widgets"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_widget_that_is_not_public_here_answers_as_an_unknown_one() {
    let (hidden, _) = ask("/api/public/widgets/riga/data", Environment::internet()).await;
    let (unknown, _) = ask("/api/public/widgets/nope/data", Environment::internet()).await;
    assert_eq!(hidden, StatusCode::NOT_FOUND);
    assert_eq!(unknown, StatusCode::NOT_FOUND);
    let (visible, body) = ask(
        "/api/public/widgets/riga/data",
        Environment::parse("local").unwrap(),
    )
    .await;
    assert_eq!(visible, StatusCode::OK);
    assert_eq!(body["data"]["temperature"], 12.0);
}

#[tokio::test]
async fn environment_names_stay_inside() {
    let (_, body) = ask(PublicFeature::PORTAL_PATH, Environment::internet()).await;
    assert_eq!(body["detected"], "internet");
    assert_eq!(body["switchable"], false);
    assert!(body.get("environments").is_none());
    let text = body.to_string();
    assert!(!text.contains("\"local\""));
    assert!(!text.contains("\"vpn\""));
}

#[tokio::test]
async fn a_visitor_at_home_without_a_session_may_switch() {
    let (_, body) = ask(
        PublicFeature::PORTAL_PATH,
        Environment::parse("local").unwrap(),
    )
    .await;
    assert_eq!(body["detected"], "local");
    assert_eq!(body["switchable"], true);
    assert_eq!(body["environments"], json!(["local", "vpn", "internet"]));
}

#[test]
fn a_public_service_has_no_field_for_anything_private() {
    let service = PublicService {
        id: "nas".into(),
        name: "NAS".into(),
        address: "http://192.168.1.10".into(),
        group: None,
        icon: None,
        description: None,
        status: None,
    };
    let json = serde_json::to_value(&service).unwrap();
    let keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        vec![
            "address",
            "description",
            "group",
            "icon",
            "id",
            "name",
            "status"
        ]
    );
    for private in [
        "url",
        "addresses",
        "probe",
        "notify",
        "public",
        "public_status",
        "environments",
    ] {
        assert!(
            !keys.contains(&private),
            "{private} must not be part of the public shape"
        );
    }
}
