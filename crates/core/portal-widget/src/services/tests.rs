use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use portal_config::ConfigStore;
use portal_feature::{ApiError, FieldError, WidgetProvider};
use portal_model::Environment;
use serde_json::{Value, json};
use tempfile::TempDir;

use super::WidgetRegistry;
use portal_feature::WidgetProblem;

pub struct Counting {
    pub calls: AtomicUsize,
    pub failing: AtomicBool,
    pub refresh: Duration,
    pub delay: Duration,
}

impl Counting {
    pub fn new() -> Arc<Counting> {
        Arc::new(Counting {
            calls: AtomicUsize::new(0),
            failing: AtomicBool::new(false),
            refresh: Duration::from_secs(60),
            delay: Duration::ZERO,
        })
    }
}

#[async_trait]
impl WidgetProvider for Counting {
    fn kind(&self) -> &'static str {
        "counter"
    }

    fn refresh(&self) -> Duration {
        self.refresh
    }

    fn check(&self, settings: &Value) -> Vec<FieldError> {
        match settings.get("city").and_then(Value::as_str) {
            Some(_) | None if settings.get("city").is_none_or(Value::is_string) => Vec::new(),
            _ => vec![FieldError::new("city", "must be a name")],
        }
    }

    async fn data(&self, _settings: &Value) -> Result<Value, WidgetProblem> {
        tokio::time::sleep(self.delay).await;
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self.failing.load(Ordering::SeqCst) {
            return Err(WidgetProblem::new("upstream is away"));
        }
        Ok(json!({ "call": call }))
    }
}

fn registry_with(text: &str, provider: Arc<Counting>) -> (TempDir, WidgetRegistry) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    (directory, WidgetRegistry::new(store, vec![provider]))
}

const ONE: &str = "[[dashboard.widgets]]\ntype = \"counter\"\nid = \"first\"\n";

#[tokio::test]
async fn data_is_cached_until_the_provider_asks_for_a_refresh() {
    let provider = Counting::new();
    let (_directory, registry) = registry_with(ONE, provider.clone());
    let first = registry
        .data("first", &Environment::internet())
        .await
        .unwrap();
    let second = registry
        .data("first", &Environment::internet())
        .await
        .unwrap();
    assert_eq!(first.data["call"], 1);
    assert_eq!(second.data["call"], 1);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(first.refresh_seconds, 60);
    assert!(!first.stale);
}

#[tokio::test]
async fn readers_arriving_together_share_one_refresh() {
    let provider = Arc::new(Counting {
        calls: AtomicUsize::new(0),
        failing: AtomicBool::new(false),
        refresh: Duration::from_secs(60),
        delay: Duration::from_millis(150),
    });
    let (_directory, registry) = registry_with(ONE, provider.clone());
    let registry = Arc::new(registry);
    let mut readers = Vec::new();
    for _ in 0..3 {
        let registry = registry.clone();
        readers.push(tokio::spawn(async move {
            registry
                .data("first", &Environment::internet())
                .await
                .unwrap()
        }));
    }
    for reader in readers {
        assert_eq!(reader.await.unwrap().data["call"], 1);
    }
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_failing_refresh_keeps_the_last_good_value_and_marks_it_stale() {
    let provider = Arc::new(Counting {
        calls: AtomicUsize::new(0),
        failing: AtomicBool::new(false),
        refresh: Duration::from_millis(1),
        delay: Duration::ZERO,
    });
    let (_directory, registry) = registry_with(ONE, provider.clone());
    let good = registry
        .data("first", &Environment::internet())
        .await
        .unwrap();
    assert_eq!(good.data["call"], 1);
    provider.failing.store(true, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(5)).await;
    let stale = registry
        .data("first", &Environment::internet())
        .await
        .unwrap();
    assert_eq!(stale.data["call"], 1);
    assert!(stale.stale);
    assert_eq!(stale.problem.as_deref(), Some("upstream is away"));
}

#[tokio::test]
async fn a_provider_that_never_succeeded_answers_bad_gateway() {
    let provider = Arc::new(Counting {
        calls: AtomicUsize::new(0),
        failing: AtomicBool::new(true),
        refresh: Duration::from_secs(60),
        delay: Duration::ZERO,
    });
    let (_directory, registry) = registry_with(ONE, provider);
    let refused = registry.data("first", &Environment::internet()).await;
    assert!(
        matches!(refused, Err(ApiError::BadGateway(_))),
        "{refused:?}"
    );
}

#[tokio::test]
async fn an_unknown_widget_and_one_of_another_environment_are_both_not_found() {
    let text =
        "[[dashboard.widgets]]\ntype = \"counter\"\nid = \"first\"\nenvironments = [\"local\"]\n";
    let (_directory, registry) = registry_with(text, Counting::new());
    let hidden = registry.data("first", &Environment::internet()).await;
    let missing = registry.data("nope", &Environment::internet()).await;
    assert!(matches!(hidden, Err(ApiError::NotFound(_))));
    assert!(matches!(missing, Err(ApiError::NotFound(_))));
    let local = Environment::parse("local").unwrap();
    assert!(registry.data("first", &local).await.is_ok());
}

#[test]
fn a_widget_with_a_provider_needs_an_id_and_the_ids_are_unique() {
    let text = "[[dashboard.widgets]]\ntype = \"counter\"\n\n[[dashboard.widgets]]\ntype = \"counter\"\nid = \"a\"\n\n[[dashboard.widgets]]\ntype = \"counter\"\nid = \"a\"\n\n[[dashboard.widgets]]\ntype = \"weather\"\n";
    let (_directory, registry) = registry_with(text, Counting::new());
    let document = registry_document(text);
    let fields: Vec<String> = registry
        .validate(&document)
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(
        fields,
        vec!["dashboard.widgets[0].id", "dashboard.widgets[2].id"]
    );
}

#[test]
fn a_setting_the_provider_refuses_names_the_widget_and_the_setting() {
    let text =
        "[[dashboard.widgets]]\ntype = \"counter\"\nid = \"first\"\nsettings = { city = 5 }\n";
    let (_directory, registry) = registry_with(text, Counting::new());
    let errors = registry.validate(&registry_document(text));
    assert_eq!(errors[0].field, "dashboard.widgets.first.settings.city");
}

fn registry_document(text: &str) -> toml_edit::DocumentMut {
    text.parse().unwrap()
}
