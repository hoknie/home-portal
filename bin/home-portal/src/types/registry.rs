use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{Feature, Gate};
use portal_widget::WidgetRegistry;

pub struct Registry {
    pub features: Vec<Arc<dyn Feature>>,
    pub gate: Arc<dyn Gate>,
    pub configuration: Arc<ConfigStore>,
    pub widgets: Arc<WidgetRegistry>,
}
