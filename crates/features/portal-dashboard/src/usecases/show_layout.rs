use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;
use portal_model::Environment;

use crate::services::layout_view;
use crate::types::LayoutView;

#[derive(Clone)]
pub struct ShowLayout {
    configuration: Arc<ConfigStore>,
}

impl ShowLayout {
    pub fn new(configuration: Arc<ConfigStore>) -> ShowLayout {
        ShowLayout { configuration }
    }

    pub fn run(&self, filter: Option<&Environment>) -> Result<Revisioned<LayoutView>, ApiError> {
        let snapshot = self.configuration.read();
        let view = layout_view(&snapshot.document, filter)?;
        Ok(Revisioned::new(view, snapshot.revision))
    }
}
