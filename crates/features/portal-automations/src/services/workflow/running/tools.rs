use std::sync::Arc;

use portal_config::ConfigStore;

use crate::clients::HttpClient;
use crate::ports::PortalActions;
use crate::services::workflow::evaluating::SecretLookup;

#[derive(Clone)]
pub struct WorkflowTools {
    pub actions: Arc<dyn PortalActions>,
    pub http: HttpClient,
    pub secrets: SecretLookup,
}

impl WorkflowTools {
    pub fn of(
        configuration: Arc<ConfigStore>,
        actions: Arc<dyn PortalActions>,
    ) -> Result<WorkflowTools, String> {
        Ok(WorkflowTools {
            actions,
            http: HttpClient::new()?,
            secrets: Arc::new(move |key: &str| {
                configuration
                    .secret(key)
                    .map(|secret| secret.expose().to_string())
            }),
        })
    }
}
