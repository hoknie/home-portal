use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::post;
use portal_config::ConfigStore;
use portal_feature::{Action, Area, Feature, Right, Rule, WidgetProvider};

use crate::controllers::{press_widget_action, preview_widget};
use crate::providers::{CustomWidgetProvider, WidgetMemory};
use crate::types::{WidgetPorts, WidgetsState};
use crate::usecases::{PressWidgetAction, PreviewWidget};

const LAYOUT_UPDATE: &[Right] = &[Right::new(Area::Layout, Action::Update)];

pub struct WidgetsFeature {
    state: WidgetsState,
    pub(crate) custom: Arc<CustomWidgetProvider>,
}

impl WidgetsFeature {
    pub const NAME: &'static str = "widgets";
    pub const ACTION: &'static str = "/api/widgets/{id}/actions/{action}";
    pub const PREVIEW: &'static str = "/api/widgets/preview";

    pub fn new(configuration: Arc<ConfigStore>, ports: WidgetPorts) -> WidgetsFeature {
        let custom = Arc::new(CustomWidgetProvider {
            configuration,
            ports,
            memory: Arc::new(WidgetMemory::default()),
        });
        WidgetsFeature {
            state: WidgetsState {
                press: PressWidgetAction::new(custom.clone()),
                preview: PreviewWidget::new(custom.clone()),
            },
            custom,
        }
    }
}

impl Feature for WidgetsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::ACTION, post(press_widget_action))
            .route(Self::PREVIEW, post(preview_widget))
            .with_state(self.state.clone())
    }

    fn widget_providers(&self) -> Vec<Arc<dyn WidgetProvider>> {
        vec![self.custom.clone()]
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::signed(Method::POST, Self::ACTION),
            Rule::needs(Method::POST, Self::PREVIEW, LAYOUT_UPDATE),
        ]
    }
}
