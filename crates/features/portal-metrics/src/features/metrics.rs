use std::sync::Arc;

use axum::Router;
use portal_feature::{Feature, WidgetProvider};

use crate::providers::HostMetricsProvider;

pub struct MetricsFeature {
    provider: Arc<HostMetricsProvider>,
}

impl MetricsFeature {
    pub const NAME: &'static str = "metrics";

    pub fn new() -> MetricsFeature {
        MetricsFeature {
            provider: Arc::new(HostMetricsProvider::new()),
        }
    }
}

impl Feature for MetricsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
    }

    fn widget_providers(&self) -> Vec<Arc<dyn WidgetProvider>> {
        vec![self.provider.clone()]
    }
}

impl Default for MetricsFeature {
    fn default() -> MetricsFeature {
        MetricsFeature::new()
    }
}
