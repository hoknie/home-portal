use toml_edit::DocumentMut;

use crate::services::{read_environments, read_network};
use crate::types::NetworkReading;

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkOfDocument;

impl NetworkOfDocument {
    pub fn run(&self, document: &DocumentMut) -> NetworkReading {
        NetworkReading {
            settings: read_network(document),
            environments: read_environments(document),
        }
    }
}
