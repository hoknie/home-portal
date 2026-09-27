use std::sync::Arc;

use portal_feature::{ApiError, FieldError};
use portal_model::Environment;
use toml_edit::DocumentMut;

use super::{StatusBoard, Supervisor};
use super::{check_entry, known_of};
use crate::ports::Publishing;
use crate::types::{ServiceEntry, ServicesSection, ShownService};

#[derive(Clone)]
pub struct Showcase {
    pub board: Arc<StatusBoard>,
    pub supervisor: Arc<Supervisor>,
    pub publishing: Arc<dyn Publishing>,
}

impl Showcase {
    pub fn shown(&self, entry: ServiceEntry) -> ShownService {
        ShownService {
            status: self.board.status(&entry.id),
            host: self.supervisor.host().clone(),
            publishing: self.publishing.https_port(),
            entry,
        }
    }

    pub fn listed(
        &self,
        document: &DocumentMut,
        environment: &Environment,
    ) -> Result<Vec<ShownService>, ApiError> {
        let section = ServicesSection::read(document)
            .map_err(|message| ApiError::Internal(format!("services section: {message}")))?;
        Ok(section
            .services
            .into_iter()
            .filter(|entry| entry.visible_to(environment))
            .map(|entry| self.shown(entry))
            .collect())
    }

    pub fn checked(
        &self,
        entry: ServiceEntry,
        document: &DocumentMut,
    ) -> Result<ServiceEntry, ApiError> {
        let mut errors: Vec<FieldError> = check_entry(&entry, &known_of(document));
        if let Some(publication) = &entry.proxy {
            errors.extend(self.publishing.problems(publication));
        }
        if errors.is_empty() {
            Ok(entry)
        } else {
            Err(ApiError::Invalid(errors))
        }
    }
}
