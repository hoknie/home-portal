use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, PoisonError};

use portal_config::ConfigStore;
use portal_feature::{ApiError, Check, FieldError, WidgetProvider};
use portal_model::Environment;
use time::OffsetDateTime;
use toml_edit::DocumentMut;

use super::validation::check_instances;
use crate::types::{Cached, Layout, WidgetData, WidgetInstance, WidgetsSection};

pub struct WidgetRegistry {
    configuration: Arc<ConfigStore>,
    providers: BTreeMap<&'static str, Arc<dyn WidgetProvider>>,
    cache: Mutex<HashMap<String, Cached>>,
    refreshing: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

impl WidgetRegistry {
    pub const UNKNOWN_WIDGET: &'static str = "no such widget";

    pub fn new(
        configuration: Arc<ConfigStore>,
        providers: Vec<Arc<dyn WidgetProvider>>,
    ) -> WidgetRegistry {
        WidgetRegistry {
            configuration,
            providers: providers
                .into_iter()
                .map(|provider| (provider.kind(), provider))
                .collect(),
            cache: Mutex::new(HashMap::new()),
            refreshing: Mutex::new(HashMap::new()),
        }
    }

    pub fn kinds(&self) -> Vec<&'static str> {
        self.providers.keys().copied().collect()
    }

    pub fn instances(&self) -> Vec<WidgetInstance> {
        WidgetsSection::read(&self.configuration.read().document).unwrap_or_default()
    }

    pub fn layout(&self) -> Option<Layout> {
        WidgetsSection::layout(&self.configuration.read().document)
            .ok()
            .flatten()
    }

    pub fn validate(&self, document: &DocumentMut) -> Vec<FieldError> {
        (self.checker())(document)
    }

    pub fn checker(&self) -> Check {
        let providers = self.providers.clone();
        Arc::new(
            move |document: &DocumentMut| match WidgetsSection::read(document) {
                Err(message) => vec![FieldError::new("dashboard.widgets", message)],
                Ok(instances) => check_instances(&instances, &providers),
            },
        )
    }

    pub async fn data(&self, id: &str, environment: &Environment) -> Result<WidgetData, ApiError> {
        let instance = self
            .instances()
            .into_iter()
            .find(|instance| instance.id.as_deref() == Some(id) && instance.visible_to(environment))
            .ok_or(ApiError::NotFound(Self::UNKNOWN_WIDGET))?;
        let provider = self
            .providers
            .get(instance.kind.as_str())
            .ok_or(ApiError::NotFound(Self::UNKNOWN_WIDGET))?
            .clone();
        let refresh = provider.refresh();
        if let Some(fresh) = self.fresh(id, refresh) {
            return Ok(fresh);
        }
        let gate = self.gate(id);
        let _refreshing = gate.lock().await;
        if let Some(fresh) = self.fresh(id, refresh) {
            return Ok(fresh);
        }
        let now = OffsetDateTime::now_utc();
        match provider.data(&instance.settings).await {
            Ok(data) => {
                let cached = Cached {
                    data,
                    fetched_at: now,
                    problem: None,
                };
                self.remember(id, cached.clone());
                Ok(answer(cached, refresh, false))
            }
            Err(problem) => {
                tracing::warn!(widget = id, %problem, "widget data could not be refreshed");
                match self.remembered(id) {
                    Some(mut cached) => {
                        cached.problem = Some(problem.to_string());
                        self.remember(id, cached.clone());
                        Ok(answer(cached, refresh, true))
                    }
                    None => Err(ApiError::BadGateway(problem.to_string())),
                }
            }
        }
    }

    fn fresh(&self, id: &str, refresh: std::time::Duration) -> Option<WidgetData> {
        let cached = self.remembered(id)?;
        let age = OffsetDateTime::now_utc() - cached.fetched_at;
        let within = age.whole_milliseconds()
            < i128::from(refresh.as_millis().min(i128::MAX as u128) as i64);
        let stale = cached.problem.is_some();
        within.then(|| answer(cached, refresh, stale))
    }

    fn remembered(&self, id: &str) -> Option<Cached> {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    fn remember(&self, id: &str, cached: Cached) {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string(), cached);
    }

    fn gate(&self, id: &str) -> Arc<tokio::sync::Mutex<()>> {
        self.refreshing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }
}

fn answer(cached: Cached, refresh: std::time::Duration, stale: bool) -> WidgetData {
    WidgetData {
        data: cached.data,
        fetched_at: cached.fetched_at,
        stale,
        problem: cached.problem,
        refresh_seconds: refresh.as_secs().max(1),
    }
}
