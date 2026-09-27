use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};

use crate::loops::CaddySync;
use crate::types::ProxyView;

#[derive(Clone)]
pub struct ShowProxy {
    configuration: Arc<ConfigStore>,
    sync: Arc<CaddySync>,
}

impl ShowProxy {
    pub fn new(configuration: Arc<ConfigStore>, sync: Arc<CaddySync>) -> ShowProxy {
        ShowProxy {
            configuration,
            sync,
        }
    }

    pub fn run(&self) -> Revisioned<ProxyView> {
        let revision = self.configuration.read().revision;
        Revisioned::new(self.sync.view(), revision)
    }
}
