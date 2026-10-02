use std::sync::Arc;

use crate::ports::{WidgetReferences, WidgetRuns, WidgetTemplates};

#[derive(Clone)]
pub struct WidgetPorts {
    pub templates: Arc<dyn WidgetTemplates>,
    pub references: Arc<dyn WidgetReferences>,
    pub runs: Arc<dyn WidgetRuns>,
}
