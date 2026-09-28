use async_trait::async_trait;

use crate::types::{PortalState, ProbeResult, StatusResult};

#[async_trait]
pub trait PortalActions: Send + Sync {
    async fn probe(&self, service: &str) -> Result<ProbeResult, String>;

    async fn status(&self, service: &str) -> Result<StatusResult, String>;

    async fn state(&self) -> Result<PortalState, String>;

    async fn notify(
        &self,
        channel: Option<&str>,
        title: &str,
        text: &str,
    ) -> Result<String, String>;
}
