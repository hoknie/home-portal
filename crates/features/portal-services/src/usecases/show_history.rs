use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::ApiError;
use portal_model::Environment;
use time::OffsetDateTime;

use crate::services::StatusBoard;
use crate::types::{HistoryRange, HistoryView, ServiceEntry, ServicesSection};

#[derive(Clone)]
pub struct ShowHistory {
    configuration: Arc<ConfigStore>,
    board: Arc<StatusBoard>,
}

impl ShowHistory {
    pub fn new(configuration: Arc<ConfigStore>, board: Arc<StatusBoard>) -> ShowHistory {
        ShowHistory {
            configuration,
            board,
        }
    }

    pub fn run(
        &self,
        id: &str,
        range: HistoryRange,
        environment: &Environment,
    ) -> Result<HistoryView, ApiError> {
        let document = self.configuration.read().document;
        if ServicesSection::visible(&document, id, environment).is_none() {
            return Err(ApiError::NotFound(ServiceEntry::UNKNOWN));
        }
        Ok(self.board.history(id, range, OffsetDateTime::now_utc()))
    }
}
