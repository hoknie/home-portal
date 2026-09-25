use std::collections::BTreeMap;

use super::{SecretString, Snapshot, Source};

pub struct Loaded {
    pub sources: Vec<Source>,
    pub snapshot: Snapshot,
    pub secrets: BTreeMap<String, SecretString>,
}
