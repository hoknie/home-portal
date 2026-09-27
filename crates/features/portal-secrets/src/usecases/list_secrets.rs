use std::collections::BTreeSet;
use std::sync::Arc;

use portal_config::ConfigStore;

use crate::services::referenced_secrets;
use crate::types::SecretEntry;

#[derive(Clone)]
pub struct ListSecrets {
    configuration: Arc<ConfigStore>,
}

impl ListSecrets {
    pub fn new(configuration: Arc<ConfigStore>) -> ListSecrets {
        ListSecrets { configuration }
    }

    pub fn run(&self) -> Vec<SecretEntry> {
        let snapshot = self.configuration.read();
        let defined: BTreeSet<String> = self.configuration.secret_names().into_iter().collect();
        let mut names = referenced_secrets(&snapshot.document);
        names.extend(defined.iter().cloned());
        names
            .into_iter()
            .map(|name| SecretEntry {
                set: defined.contains(&name),
                name,
            })
            .collect()
    }
}
