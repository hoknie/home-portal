use std::path::Path;
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::loops::DnsRuntime;
use crate::repositories::{SECTION, write_dns};
use crate::types::{DnsChoice, DnsView};

#[derive(Clone)]
pub struct ChangeDns {
    configuration: Arc<ConfigStore>,
    runtime: Arc<DnsRuntime>,
}

impl ChangeDns {
    pub fn new(configuration: Arc<ConfigStore>, runtime: Arc<DnsRuntime>) -> ChangeDns {
        ChangeDns {
            configuration,
            runtime,
        }
    }

    pub async fn run(
        &self,
        choice: &DnsChoice,
        revision: &Revision,
    ) -> Result<Revisioned<DnsView>, ApiError> {
        let target = self
            .configuration
            .read()
            .origins
            .table(SECTION)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.configuration.writes_to());
        self.configuration
            .update(&target, revision, |document| {
                write_dns(document, choice);
                Ok(())
            })
            .await?;
        self.runtime.refresh_now();
        let library = &self.runtime.library;
        let view = DnsView {
            settings: library.settings(),
            book: library.book(),
            state: library.state(),
        };
        Ok(Revisioned::new(view, self.configuration.read().revision))
    }
}
