use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use portal_feature::{FieldError, WidgetProblem, WidgetProvider};
use serde_json::Value;

use crate::services::HostReader;
use crate::types::MetricsSettings;

pub struct HostMetricsProvider {
    reader: Arc<HostReader>,
}

impl HostMetricsProvider {
    pub const KIND: &'static str = "host-metrics";
    pub const REFRESH: Duration = Duration::from_secs(5);

    pub fn new() -> HostMetricsProvider {
        HostMetricsProvider {
            reader: Arc::new(HostReader::new()),
        }
    }

    fn settings(settings: &Value) -> Result<MetricsSettings, String> {
        serde_json::from_value(settings.clone()).map_err(|error| error.to_string())
    }
}

#[async_trait]
impl WidgetProvider for HostMetricsProvider {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn refresh(&self) -> Duration {
        Self::REFRESH
    }

    fn check(&self, settings: &Value) -> Vec<FieldError> {
        match Self::settings(settings) {
            Ok(_) => Vec::new(),
            Err(message) => vec![FieldError::new("disks", message)],
        }
    }

    async fn data(&self, settings: &Value) -> Result<Value, WidgetProblem> {
        let settings = Self::settings(settings).map_err(WidgetProblem::new)?;
        let reader = self.reader.clone();
        let reading = tokio::task::spawn_blocking(move || reader.read(&settings))
            .await
            .map_err(|error| WidgetProblem::new(format!("reading the host failed: {error}")))?;
        serde_json::to_value(reading).map_err(|error| WidgetProblem::new(error.to_string()))
    }
}

impl Default for HostMetricsProvider {
    fn default() -> HostMetricsProvider {
        HostMetricsProvider::new()
    }
}
