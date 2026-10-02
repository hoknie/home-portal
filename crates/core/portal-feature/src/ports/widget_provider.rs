use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::types::{FieldError, WidgetLimits, WidgetNeed, WidgetProblem};

#[async_trait]
pub trait WidgetProvider: Send + Sync {
    fn kind(&self) -> &'static str;

    fn refresh(&self) -> Duration;

    fn check(&self, settings: &Value) -> Vec<FieldError>;

    async fn data(&self, settings: &Value) -> Result<Value, WidgetProblem>;

    async fn data_of(&self, _id: &str, settings: &Value) -> Result<Value, WidgetProblem> {
        self.data(settings).await
    }

    fn limits(&self, _settings: &Value) -> WidgetLimits {
        WidgetLimits::of(self.refresh())
    }

    fn needs(&self, _settings: &Value) -> Vec<WidgetNeed> {
        Vec::new()
    }

    fn expired(&self, _id: &str) -> bool {
        false
    }

    fn available(&self) -> bool {
        true
    }

    fn public_settings(&self, _settings: &Value) -> Value {
        Value::Object(serde_json::Map::new())
    }

    fn public_data(&self, data: Value) -> Value {
        data
    }
}
