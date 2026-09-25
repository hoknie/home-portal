use std::sync::Arc;

use async_trait::async_trait;
use portal_feature::ApiError;
use portal_model::Environment;
use portal_public::{PublicLayout, PublicSection, PublicWidget};
use portal_widget::{WidgetData, WidgetRegistry};

pub struct WidgetLayout {
    pub widgets: Arc<WidgetRegistry>,
}

#[async_trait]
impl PublicLayout for WidgetLayout {
    async fn public_widgets(&self, environment: &Environment) -> Vec<PublicWidget> {
        self.widgets
            .layout()
            .map(|layout| layout.widgets)
            .unwrap_or_default()
            .into_iter()
            .filter(|instance| instance.public_in(environment))
            .map(|instance| PublicWidget {
                kind: instance.kind,
                id: instance.id,
                title: instance.title,
                settings: instance.settings,
                section: instance.section,
                size: instance.size,
            })
            .collect()
    }

    async fn public_sections(&self, environment: &Environment) -> Vec<PublicSection> {
        let Some(layout) = self.widgets.layout() else {
            return Vec::new();
        };
        layout
            .sections
            .into_iter()
            .filter(|section| {
                layout.widgets.iter().any(|widget| {
                    widget.public_in(environment) && widget.section.as_deref() == Some(&section.id)
                })
            })
            .map(|section| PublicSection {
                id: section.id,
                title: section.title,
            })
            .collect()
    }

    async fn public_data(
        &self,
        id: &str,
        environment: &Environment,
    ) -> Result<WidgetData, ApiError> {
        let public =
            self.widgets.instances().into_iter().any(|instance| {
                instance.id.as_deref() == Some(id) && instance.public_in(environment)
            });
        if !public {
            return Err(ApiError::NotFound(WidgetRegistry::UNKNOWN_WIDGET));
        }
        self.widgets.data(id, environment).await
    }
}
