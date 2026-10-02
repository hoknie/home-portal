use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use portal_config::configuration_files;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    files: Vec<(PathBuf, Option<(SystemTime, u64)>)>,
}

impl Fingerprint {
    pub fn of(main: &Path) -> Fingerprint {
        Fingerprint {
            files: configuration_files(main)
                .into_iter()
                .map(|path| {
                    let stamp = fs::metadata(&path)
                        .ok()
                        .and_then(|metadata| Some((metadata.modified().ok()?, metadata.len())));
                    (path, stamp)
                })
                .collect(),
        }
    }
}
