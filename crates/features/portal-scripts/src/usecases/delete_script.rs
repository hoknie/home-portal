use portal_config::Revision;
use portal_feature::ApiError;

use super::ScriptFiles;

#[derive(Clone)]
pub struct DeleteScript {
    files: ScriptFiles,
}

impl DeleteScript {
    pub fn new(files: ScriptFiles) -> DeleteScript {
        DeleteScript { files }
    }

    pub fn run(&self, path: &str, expected: &Revision) -> Result<(), ApiError> {
        let parsed = ScriptFiles::path("path", path)?;
        self.files.writer.delete(&parsed, expected)
    }
}
