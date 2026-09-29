use portal_config::Revision;
use portal_feature::ApiError;

use super::ScriptFiles;
use crate::types::ScriptFile;

#[derive(Clone)]
pub struct CreateScript {
    files: ScriptFiles,
}

impl CreateScript {
    pub fn new(files: ScriptFiles) -> CreateScript {
        CreateScript { files }
    }

    pub fn run(&self, path: &str, content: &str) -> Result<(Revision, ScriptFile), ApiError> {
        let parsed = ScriptFiles::path("path", path)?;
        let revision = self.files.writer.create(&parsed, content)?;
        Ok((revision, self.files.entry(&parsed.text())?))
    }
}
