use std::sync::Arc;

use axum::Router;
use portal_feature::{Feature, WidgetProvider};

use crate::clients::OpenMeteo;
use crate::providers::WeatherProvider;

pub struct WeatherFeature {
    provider: Arc<WeatherProvider>,
}

impl WeatherFeature {
    pub const NAME: &'static str = "weather";
    pub const PORTAL_TIMEZONE: &'static str = "auto";

    pub fn new(timezone: &str) -> Result<WeatherFeature, String> {
        Ok(WeatherFeature {
            provider: Arc::new(WeatherProvider::new(OpenMeteo::ENDPOINT, timezone)?),
        })
    }
}

impl Feature for WeatherFeature {
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
