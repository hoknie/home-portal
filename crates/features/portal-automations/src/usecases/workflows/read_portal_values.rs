use std::sync::Arc;

use portal_feature::ApiError;
use serde_json::Value;

use crate::ports::PortalActions;
use crate::services::AutomationSink;

#[derive(Clone)]
pub struct ReadPortalValues {
    actions: Arc<dyn PortalActions>,
    sink: Arc<AutomationSink>,
}

impl ReadPortalValues {
    pub fn new(actions: Arc<dyn PortalActions>, sink: Arc<AutomationSink>) -> ReadPortalValues {
        ReadPortalValues { actions, sink }
    }

    pub async fn run(&self) -> Result<Value, ApiError> {
        let state = self
            .actions
            .state()
            .await
            .map_err(ApiError::ServiceUnavailable)?;
        Ok(state.value_with(self.sink.cache.switches()))
    }
}
