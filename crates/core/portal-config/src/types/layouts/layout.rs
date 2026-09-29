use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Home, Section};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    homes: BTreeMap<Section, Home>,
}

impl Layout {
    pub const WORKFLOW_EXTENSION: &'static str = "toml";

    pub fn new(homes: BTreeMap<Section, Home>) -> Layout {
        Layout { homes }
    }

    pub fn home(&self, section: Section) -> &Home {
        &self.homes[&section]
    }

    pub fn file_of(&self, section: Section) -> PathBuf {
        self.home(section).path().to_path_buf()
    }

    pub fn folder(&self) -> PathBuf {
        self.file_of(Section::Workflows)
    }

    pub fn workflow_file(&self, id: &str) -> PathBuf {
        self.folder()
            .join(format!("{id}.{}", Self::WORKFLOW_EXTENSION))
    }

    pub fn files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = Vec::new();
        for section in Section::ALL {
            if let Home::File(path) = self.home(section)
                && !files.contains(path)
            {
                files.push(path.clone());
            }
        }
        files
    }

    pub fn in_folder(&self, path: &Path) -> bool {
        path.parent() == Some(self.folder().as_path())
    }
}
