use super::{Snapshot, Source, Stamp};

pub struct Current {
    pub sources: Vec<Source>,
    pub snapshot: Snapshot,
    pub stamps: Vec<Option<Stamp>>,
    pub problem: Option<String>,
}
