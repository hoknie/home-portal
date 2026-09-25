use async_trait::async_trait;
use portal_feature::ApiError;
use portal_model::Environment;
use portal_widget::WidgetData;

use crate::responses::{PublicSection, PublicWidget};

#[async_trait]
pub trait PublicLayout: Send + Sync {
    async fn public_widgets(&self, environment: &Environment) -> Vec<PublicWidget>;

    async fn public_sections(&self, environment: &Environment) -> Vec<PublicSection>;

    async fn public_data(
        &self,
        id: &str,
        environment: &Environment,
    ) -> Result<WidgetData, ApiError>;
}
