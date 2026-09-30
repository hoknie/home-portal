use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, PermissionCode};

pub struct FolderCheck {
    pub name: String,
    pub path: PathBuf,
}

impl PermissionCheck for FolderCheck {
    fn code(&self) -> PermissionCode {
        PermissionCode::Folder(self.name.clone())
    }

    fn ask(&self) -> Finding {
        listed(&self.path)
    }
}

pub fn listed(path: &Path) -> Finding {
    match fs::read_dir(path) {
        Ok(_) => Finding::granted(),
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            Finding::denied(Advice::AllowInSettings)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Finding::not_applicable(None),
        Err(_) => Finding::unknown(Advice::CheckFailed),
    }
}
