use std::path::PathBuf;
use std::sync::Arc;

use portal_config::{ConfigStore, Storage};
use portal_feature::ApiError;

use super::ListScripts;
use crate::ports::ProcessIdentity;
use crate::services::{ScriptWriter, folder_name, script_path};
use crate::types::ScriptFile;

#[derive(Clone)]
pub struct ScriptFiles {
    pub writer: ScriptWriter,
    pub list: ListScripts,
}

impl ScriptFiles {
    pub fn new(configuration: &ConfigStore, identity: Arc<dyn ProcessIdentity>) -> ScriptFiles {
        ScriptFiles::at(configuration.storage(Storage::Scripts), identity)
    }

    pub fn at(root: PathBuf, identity: Arc<dyn ProcessIdentity>) -> ScriptFiles {
        ScriptFiles {
            writer: ScriptWriter::at(root.clone()),
            list: ListScripts::at(root, identity),
        }
    }

    pub fn entry(&self, path: &str) -> Result<ScriptFile, ApiError> {
        self.list
            .tree()
            .and_then(|tree| tree.files.into_iter().find(|file| file.path == path))
            .ok_or(ApiError::NotFound("no such script"))
    }

    pub fn path(field: &str, text: &str) -> Result<portal_model::ScriptPath, ApiError> {
        script_path(field, text)
    }

    pub fn folder(text: &str) -> Result<String, ApiError> {
        folder_name("name", text)
    }
}
