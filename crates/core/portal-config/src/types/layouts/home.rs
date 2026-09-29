use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Home {
    File(PathBuf),
    Folder(PathBuf),
}

impl Home {
    pub fn path(&self) -> &Path {
        match self {
            Home::File(path) | Home::Folder(path) => path,
        }
    }
}
