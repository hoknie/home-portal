use portal_feature::ApiError;

use super::ScriptFiles;

#[derive(Clone)]
pub struct CreateFolder {
    files: ScriptFiles,
}

impl CreateFolder {
    pub fn new(files: ScriptFiles) -> CreateFolder {
        CreateFolder { files }
    }

    pub fn run(&self, name: Option<&str>) -> Result<(), ApiError> {
        let name = name.map(ScriptFiles::folder).transpose()?;
        self.files.writer.create_folder(name.as_deref())
    }
}

#[derive(Clone)]
pub struct DeleteFolder {
    files: ScriptFiles,
}

impl DeleteFolder {
    pub fn new(files: ScriptFiles) -> DeleteFolder {
        DeleteFolder { files }
    }

    pub fn run(&self, name: &str) -> Result<(), ApiError> {
        self.files.writer.delete_folder(&ScriptFiles::folder(name)?)
    }
}
