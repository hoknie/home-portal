use portal_config::Revision;
use portal_feature::ApiError;

use super::ScriptFiles;
use crate::types::ScriptFile;

#[derive(Clone)]
pub struct WriteScript {
    files: ScriptFiles,
}

impl WriteScript {
    pub fn new(files: ScriptFiles) -> WriteScript {
        WriteScript { files }
    }

    pub fn run(
        &self,
        path: &str,
        content: &str,
        expected: &Revision,
    ) -> Result<(Revision, ScriptFile), ApiError> {
        let parsed = ScriptFiles::path("path", path)?;
        let revision = self.files.writer.replace(&parsed, content, expected)?;
        Ok((revision, self.files.entry(&parsed.text())?))
    }
}
