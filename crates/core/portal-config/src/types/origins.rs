use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Origins {
    entries: BTreeMap<String, Vec<PathBuf>>,
    tables: BTreeMap<String, PathBuf>,
}

impl Origins {
    pub fn record(&mut self, section: &str, path: &Path) {
        self.entries
            .entry(section.to_string())
            .or_default()
            .push(path.to_path_buf());
    }

    pub fn record_table(&mut self, dotted: &str, path: &Path) {
        self.tables
            .entry(dotted.to_string())
            .or_insert_with(|| path.to_path_buf());
    }

    pub fn of(&self, section: &str, index: usize) -> Option<&Path> {
        self.entries.get(section)?.get(index).map(PathBuf::as_path)
    }

    pub fn table(&self, dotted: &str) -> Option<&Path> {
        self.tables.get(dotted).map(PathBuf::as_path)
    }

    pub fn count(&self, section: &str) -> usize {
        self.entries.get(section).map_or(0, Vec::len)
    }
}
