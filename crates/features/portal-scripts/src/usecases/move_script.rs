use portal_config::Revision;
use portal_feature::ApiError;

use super::ScriptFiles;
use crate::types::ScriptFile;

#[derive(Clone)]
pub struct MoveScript {
    files: ScriptFiles,
}

impl MoveScript {
    pub fn new(files: ScriptFiles) -> MoveScript {
        MoveScript { files }
    }

    pub fn run(
        &self,
        from: &str,
        to: &str,
        expected: &Revision,
    ) -> Result<(Revision, ScriptFile), ApiError> {
        let source = ScriptFiles::path("from", from)?;
        let target = ScriptFiles::path("to", to)?;
        let revision = self.files.writer.rename(&source, &target, expected)?;
        Ok((revision, self.files.entry(&target.text())?))
    }
}
