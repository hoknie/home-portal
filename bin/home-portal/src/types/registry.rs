use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{EventSink, Feature, Gate};
use portal_web::AssetSource;
use portal_widget::WidgetRegistry;

use super::Restart;

pub struct Registry {
    pub features: Vec<Arc<dyn Feature>>,
    pub gate: Arc<dyn Gate>,
    pub configuration: Arc<ConfigStore>,
    pub widgets: Arc<WidgetRegistry>,
    pub events: Arc<dyn EventSink>,
    pub restart: Restart,
    pub interface: Arc<dyn AssetSource>,
}
