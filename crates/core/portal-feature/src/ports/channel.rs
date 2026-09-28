use async_trait::async_trait;
use serde_json::Value;
use toml_edit::{DocumentMut, Table};

use super::SecretSource;
use crate::types::{ChannelReadiness, FieldError, Notification};

#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &'static str;

    fn problems(&self, document: &DocumentMut, secrets: &dyn SecretSource) -> Vec<FieldError>;

    fn readiness(&self, document: &DocumentMut, secrets: &dyn SecretSource) -> ChannelReadiness;

    fn settings(&self, document: &DocumentMut) -> Value;

    fn apply(&self, table: &mut Table, settings: &Value) -> Result<(), Vec<FieldError>>;

    async fn deliver(
        &self,
        notification: &Notification,
        document: &DocumentMut,
        secrets: &dyn SecretSource,
    ) -> Result<(), String>;
}
