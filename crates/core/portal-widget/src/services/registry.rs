use std::collections::BTreeMap;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{ApiError, Check, FieldError, WidgetLimits, WidgetNeed, WidgetProvider};
use portal_model::Environment;
use time::OffsetDateTime;
use toml_edit::DocumentMut;

use super::flights::{Flights, Outcome};
use super::validation::check_instances;
use crate::types::{Cached, Layout, WidgetAnswer, WidgetData, WidgetInstance, WidgetsSection};

pub struct WidgetRegistry {
    configuration: Arc<ConfigStore>,
    providers: BTreeMap<&'static str, Arc<dyn WidgetProvider>>,
    flights: Arc<Flights>,
}

impl WidgetRegistry {
    pub const UNKNOWN_WIDGET: &'static str = "no such widget";
    pub const LOST: &'static str = "the refresh of the data was lost";

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
            flights: Arc::new(Flights::default()),
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
            move |document: &DocumentMut| match WidgetsSection::definitions(document) {
                Err(message) => vec![FieldError::new("dashboard", message)],
                Ok(definitions) => check_instances(&definitions, &providers),
            },
        )
    }

    pub async fn data(
        &self,
        id: &str,
        environment: &Environment,
    ) -> Result<WidgetAnswer, ApiError> {
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
        if !provider.available() {
            return Err(ApiError::NotFound(Self::UNKNOWN_WIDGET));
        }
        let limits = provider.limits(&instance.settings);
        let refresh = limits.refresh;
        if !provider.expired(id)
            && let Some(fresh) = self.fresh(id, refresh)
        {
            return Ok(WidgetAnswer::Ready(fresh));
        }
        let mut landing = self
            .flights
            .start(id, provider, instance.settings.clone(), limits);
        let waited = tokio::time::timeout(
            WidgetLimits::LONGEST_WAIT,
            landing.wait_for(Option::is_some),
        )
        .await
        .map(|landed| landed.map(|outcome| outcome.clone()));
        let remembered = self.flights.remembered(id);
        match (waited, remembered) {
            (Ok(Ok(Some(Outcome::Done))), Some(cached)) => {
                Ok(WidgetAnswer::Ready(answer(cached, refresh, false, false)))
            }
            (Ok(Ok(Some(Outcome::Failed(_)))), Some(cached)) => {
                Ok(WidgetAnswer::Ready(answer(cached, refresh, true, false)))
            }
            (Ok(Ok(Some(Outcome::Failed(problem)))), None) => Err(ApiError::BadGateway(problem)),
            (Err(_), Some(cached)) => {
                let stale = cached.problem.is_some();
                Ok(WidgetAnswer::Ready(answer(cached, refresh, stale, true)))
            }
            (Err(_), None) => Ok(WidgetAnswer::Refreshing),
            (Ok(_), _) => Err(ApiError::BadGateway(Self::LOST.to_string())),
        }
    }

    pub async fn public_data(
        &self,
        id: &str,
        environment: &Environment,
    ) -> Result<WidgetAnswer, ApiError> {
        let kind = self
            .instances()
            .into_iter()
            .find(|instance| instance.id.as_deref() == Some(id))
            .map(|instance| instance.kind);
        let answered = self.data(id, environment).await?;
        let Some(provider) = kind.and_then(|kind| self.providers.get(kind.as_str()).cloned())
        else {
            return Ok(answered);
        };
        Ok(answered.map(|data| WidgetData {
            data: provider.public_data(data.data),
            ..data
        }))
    }

    pub fn needs(&self, kind: &str, settings: &serde_json::Value) -> Vec<WidgetNeed> {
        self.providers
            .get(kind)
            .map(|provider| provider.needs(settings))
            .unwrap_or_default()
    }

    pub fn public_settings(&self, kind: &str, settings: &serde_json::Value) -> serde_json::Value {
        match self.providers.get(kind) {
            Some(provider) => provider.public_settings(settings),
            None => settings.clone(),
        }
    }

    fn fresh(&self, id: &str, refresh: std::time::Duration) -> Option<WidgetData> {
        let cached = self.flights.remembered(id)?;
        let age = OffsetDateTime::now_utc() - cached.fetched_at;
        let within = age.whole_milliseconds()
            < i128::from(refresh.as_millis().min(i128::MAX as u128) as i64);
        let stale = cached.problem.is_some();
        within.then(|| answer(cached, refresh, stale, false))
    }
}

fn answer(
    cached: Cached,
    refresh: std::time::Duration,
    stale: bool,
    refreshing: bool,
) -> WidgetData {
    WidgetData {
        data: cached.data,
        fetched_at: cached.fetched_at,
        stale,
        problem: cached.problem,
        refresh_seconds: refresh.as_secs().max(1),
        refreshing,
    }
}
