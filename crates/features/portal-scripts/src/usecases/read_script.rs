use portal_config::Revision;
use portal_feature::ApiError;

use super::ScriptFiles;
use crate::types::ScriptFile;

#[derive(Clone)]
pub struct ReadScript {
    files: ScriptFiles,
}

impl ReadScript {
    pub fn new(files: ScriptFiles) -> ReadScript {
        ReadScript { files }
    }

    pub fn run(&self, path: &str) -> Result<(String, Revision, ScriptFile), ApiError> {
        let parsed = ScriptFiles::path("path", path)?;
        let (text, revision) = self.files.writer.read(&parsed)?;
        Ok((text, revision, self.files.entry(&parsed.text())?))
    }
}
