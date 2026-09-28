use std::sync::Arc;

use portal_config::ConfigStore;
use portal_model::ProbeOutcome;
use time::OffsetDateTime;

use crate::services::{Showcase, probe_once};
use crate::types::ServicesSection;

#[derive(Clone)]
pub struct ProbeService {
    configuration: Arc<ConfigStore>,
    showcase: Showcase,
}

impl ProbeService {
    pub fn new(configuration: Arc<ConfigStore>, showcase: Showcase) -> ProbeService {
        ProbeService {
            configuration,
            showcase,
        }
    }

    pub async fn run(&self, id: &str) -> Result<ProbeOutcome, String> {
        let entry = ServicesSection::read(&self.configuration.read().document)?
            .services
            .into_iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| format!("no service {id}"))?;
        let report = probe_once(&entry, self.showcase.supervisor.host()).await?;
        self.showcase
            .board
            .record(&entry, report.outcome.clone(), OffsetDateTime::now_utc());
        Ok(report.outcome)
    }
}
