use std::sync::Arc;

use axum::Router;
use portal_config::ConfigStore;
use portal_feature::{Feature, WidgetProvider};

use crate::providers::CalendarProvider;

pub struct CalendarFeature {
    provider: Arc<CalendarProvider>,
}

impl CalendarFeature {
    pub const NAME: &'static str = "calendar";

    pub fn new(configuration: Arc<ConfigStore>) -> Result<CalendarFeature, String> {
        Ok(CalendarFeature {
            provider: Arc::new(CalendarProvider::new(configuration)?),
        })
    }
}

impl Feature for CalendarFeature {
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
