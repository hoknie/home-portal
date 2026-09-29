use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use portal_config::Revision;
use portal_model::ScriptHeader;

use crate::types::{ProblemCode, ScriptFile, ScriptProblem};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Glance {
    pub header: ScriptHeader,
    pub binary: bool,
    pub revision: Option<String>,
}

#[derive(Default)]
pub struct HeaderCache {
    known: Mutex<HashMap<PathBuf, (SystemTime, u64, Glance)>>,
}

impl HeaderCache {
    pub const LARGEST: usize = 500;

    pub fn glance(&self, path: &Path, problem: Option<&ScriptProblem>) -> Glance {
        if problem.is_some_and(|problem| !Self::lies_inside(problem.code)) {
            return Glance::default();
        }
        let Ok(metadata) = fs::symlink_metadata(path) else {
            return Glance::default();
        };
        if !metadata.is_file() {
            return Glance::default();
        }
        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let size = metadata.len();
        let mut known = self.known.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((when, length, glance)) = known.get(path)
            && *when == modified
            && *length == size
        {
            return glance.clone();
        }
        let glance = Self::read(path);
        if known.len() >= Self::LARGEST {
            known.clear();
        }
        known.insert(path.to_path_buf(), (modified, size, glance.clone()));
        glance
    }

    pub fn read(path: &Path) -> Glance {
        let mut bytes = Vec::new();
        let read = File::open(path).and_then(|file| {
            file.take(ScriptFile::LARGEST_TEXT + 1)
                .read_to_end(&mut bytes)
        });
        match read {
            Ok(_) => {
                let whole = bytes.len() as u64 <= ScriptFile::LARGEST_TEXT;
                let start = &bytes[..bytes.len().min(ScriptHeader::MOST_BYTES)];
                Glance {
                    header: ScriptHeader::parse(&String::from_utf8_lossy(start)),
                    binary: Self::binary(start),
                    revision: whole.then(|| Revision::of(&bytes).to_string()),
                }
            }
            Err(_) => Glance::default(),
        }
    }

    pub fn binary(bytes: &[u8]) -> bool {
        if bytes.contains(&0) {
            return true;
        }
        match std::str::from_utf8(bytes) {
            Ok(_) => false,
            Err(error) => error.error_len().is_some(),
        }
    }

    fn lies_inside(code: ProblemCode) -> bool {
        matches!(
            code,
            ProblemCode::NotExecutable
                | ProblemCode::Writable
                | ProblemCode::Owner
                | ProblemCode::FolderWritable
                | ProblemCode::FolderOwner
        )
    }
}
