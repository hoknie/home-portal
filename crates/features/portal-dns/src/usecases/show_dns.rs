use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};

use crate::loops::DnsRuntime;
use crate::types::DnsView;

#[derive(Clone)]
pub struct ShowDns {
    configuration: Arc<ConfigStore>,
    runtime: Arc<DnsRuntime>,
}

impl ShowDns {
    pub fn new(configuration: Arc<ConfigStore>, runtime: Arc<DnsRuntime>) -> ShowDns {
        ShowDns {
            configuration,
            runtime,
        }
    }

    pub fn run(&self) -> Revisioned<DnsView> {
        let library = &self.runtime.library;
        let view = DnsView {
            settings: library.settings(),
            book: library.book(),
            state: library.state(),
        };
        Revisioned::new(view, self.configuration.read().revision)
    }
}
