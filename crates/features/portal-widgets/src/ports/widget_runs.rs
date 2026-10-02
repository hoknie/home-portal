use async_trait::async_trait;
use portal_feature::ApiError;
use serde_json::Value;

use crate::types::WidgetSource;

pub struct Started<'a> {
    pub widget: &'a str,
    pub title: Option<&'a str>,
    pub by: &'a str,
}

#[async_trait]
pub trait WidgetRuns: Send + Sync {
    async fn run_source(
        &self,
        widget: &str,
        title: &str,
        source: &WidgetSource,
    ) -> Result<Value, String>;
    fn start_automation(
        &self,
        started: Started<'_>,
        automation: &str,
        fields: &[(String, String)],
    ) -> Result<u64, ApiError>;
    fn start_workflow(
        &self,
        started: Started<'_>,
        workflow: &str,
        inputs: Vec<(String, Value)>,
    ) -> Result<u64, ApiError>;
}
