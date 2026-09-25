use std::time::Duration;

use crate::types::FieldError;
use async_trait::async_trait;
use serde_json::Value;

use crate::types::WidgetProblem;

#[async_trait]
pub trait WidgetProvider: Send + Sync {
    fn kind(&self) -> &'static str;

    fn refresh(&self) -> Duration;

    fn check(&self, settings: &Value) -> Vec<FieldError>;

    async fn data(&self, settings: &Value) -> Result<Value, WidgetProblem>;
}
