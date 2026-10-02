use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use portal_feature::{WidgetLimits, WidgetProvider};
use serde_json::Value;
use time::OffsetDateTime;
use tokio::sync::watch;

use crate::types::Cached;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Done,
    Failed(String),
}

pub type Landing = watch::Receiver<Option<Outcome>>;

#[derive(Default)]
pub struct Flights {
    cache: Mutex<HashMap<String, Cached>>,
    flying: Mutex<HashMap<String, Landing>>,
}

impl Flights {
    pub fn remembered(&self, id: &str) -> Option<Cached> {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    pub fn remember(&self, id: &str, cached: Cached) {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string(), cached);
    }

    pub fn start(
        self: &Arc<Self>,
        id: &str,
        provider: Arc<dyn WidgetProvider>,
        settings: Value,
        limits: WidgetLimits,
    ) -> Landing {
        let mut flying = self.flying.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(landing) = flying.get(id) {
            return landing.clone();
        }
        let (landed, landing) = watch::channel(None);
        flying.insert(id.to_string(), landing.clone());
        drop(flying);
        let flights = self.clone();
        let id = id.to_string();
        tokio::spawn(async move {
            let outcome = flights
                .fetch(&id, provider.as_ref(), &settings, limits)
                .await;
            flights
                .flying
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&id);
            let _ = landed.send(Some(outcome));
        });
        landing
    }

    async fn fetch(
        &self,
        id: &str,
        provider: &dyn WidgetProvider,
        settings: &Value,
        limits: WidgetLimits,
    ) -> Outcome {
        let now = OffsetDateTime::now_utc();
        let fetched =
            match tokio::time::timeout(limits.timeout, provider.data_of(id, settings)).await {
                Ok(fetched) => fetched.map_err(|problem| problem.to_string()),
                Err(_) => Err(format!(
                    "the data took longer than {} seconds",
                    limits.timeout.as_secs()
                )),
            };
        match fetched {
            Ok(data) => {
                self.remember(
                    id,
                    Cached {
                        data,
                        fetched_at: now,
                        problem: None,
                    },
                );
                Outcome::Done
            }
            Err(problem) => {
                tracing::warn!(widget = id, %problem, "widget data could not be refreshed");
                if let Some(mut cached) = self.remembered(id) {
                    cached.problem = Some(problem.clone());
                    self.remember(id, cached);
                }
                Outcome::Failed(problem)
            }
        }
    }
}
