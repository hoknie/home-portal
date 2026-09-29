use std::path::PathBuf;

use super::{Snapshot, Source, Stamp};

pub struct Current {
    pub sources: Vec<Source>,
    pub snapshot: Snapshot,
    pub stamps: Vec<Option<Stamp>>,
    pub listing: Vec<PathBuf>,
    pub problem: Option<String>,
}
