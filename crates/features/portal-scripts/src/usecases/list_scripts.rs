use std::path::PathBuf;
use std::sync::Arc;

use portal_config::{ConfigStore, Storage};

use crate::ports::ProcessIdentity;
use crate::services::{HeaderCache, ScriptsDirectory};
use crate::types::{ScriptFile, ScriptTree, Unreadable};

#[derive(Clone)]
pub struct ListScripts {
    directory: ScriptsDirectory,
    headers: Arc<HeaderCache>,
}

impl ListScripts {
    pub fn new(configuration: &ConfigStore, identity: Arc<dyn ProcessIdentity>) -> ListScripts {
        ListScripts::at(configuration.storage(Storage::Scripts), identity)
    }

    pub fn at(root: PathBuf, identity: Arc<dyn ProcessIdentity>) -> ListScripts {
        ListScripts {
            directory: ScriptsDirectory::at(root, identity),
            headers: Arc::new(HeaderCache::default()),
        }
    }

    pub fn run(&self) -> Option<Vec<ScriptFile>> {
        self.tree().map(|tree| tree.files)
    }

    pub fn tree(&self) -> Option<ScriptTree> {
        let root = self.directory.canonical_root();
        let mut tree = self.directory.tree()?;
        for file in &mut tree.files {
            let glance = self
                .headers
                .glance(&root.join(&file.path), file.problem.as_ref());
            file.header = glance.header;
            file.revision = glance.revision;
            if glance.binary && file.unreadable.is_none() {
                file.unreadable = Some(Unreadable::Binary);
            }
        }
        Some(tree)
    }

    pub fn user(&self) -> u32 {
        self.directory.user()
    }

    pub fn root(&self) -> PathBuf {
        self.directory.canonical_root()
    }
}
