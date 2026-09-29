use std::path::PathBuf;
use std::sync::Arc;

use portal_config::{ConfigStore, Storage};

use crate::ports::ProcessIdentity;
use crate::services::ScriptsDirectory;
use crate::types::ScriptProblem;

#[derive(Clone)]
pub struct ResolveScript {
    directory: ScriptsDirectory,
}

impl ResolveScript {
    pub fn new(configuration: &ConfigStore, identity: Arc<dyn ProcessIdentity>) -> ResolveScript {
        ResolveScript::at(configuration.storage(Storage::Scripts), identity)
    }

    pub fn at(root: PathBuf, identity: Arc<dyn ProcessIdentity>) -> ResolveScript {
        ResolveScript {
            directory: ScriptsDirectory::at(root, identity),
        }
    }

    pub fn run(&self, script: &str) -> Result<PathBuf, ScriptProblem> {
        self.directory.resolve(script)
    }

    pub fn root(&self) -> PathBuf {
        self.directory.root().to_path_buf()
    }
}
