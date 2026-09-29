use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use portal_model::{ScriptHeader, ScriptPath};

use crate::ports::ScriptLibrary;
use crate::types::{Refusal, RefusalCode, ScriptEntry};

pub struct FakeScripts {
    root: PathBuf,
}

impl FakeScripts {
    const WRITABLE_BY_OTHERS: u32 = 0o022;
    const ANYONE_EXECUTES: u32 = 0o111;

    pub fn at(root: PathBuf) -> FakeScripts {
        FakeScripts { root }
    }
}

impl ScriptLibrary for FakeScripts {
    fn root(&self) -> PathBuf {
        fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone())
    }

    fn resolve(&self, script: &str) -> Result<PathBuf, Refusal> {
        let named = self.root.join(script);
        if let Some(problem) = ScriptPath::problem(script) {
            return Err(Refusal::new(RefusalCode::Shape, &named, problem));
        }
        let root = self.root();
        let resolved = fs::canonicalize(&named).map_err(|_| {
            Refusal::new(
                RefusalCode::NotFound,
                &named,
                format!("{script} does not exist in the scripts directory"),
            )
        })?;
        if !resolved.starts_with(&root) {
            return Err(Refusal::new(
                RefusalCode::Outside,
                &named,
                format!("{script} is outside the scripts directory"),
            ));
        }
        let mode = fs::metadata(&resolved)
            .map(|metadata| metadata.permissions().mode())
            .unwrap_or_default();
        if mode & Self::WRITABLE_BY_OTHERS != 0 {
            return Err(Refusal::new(
                RefusalCode::Writable,
                &resolved,
                format!("{script} can be written by group or others"),
            ));
        }
        if mode & Self::ANYONE_EXECUTES == 0 {
            return Err(Refusal::new(
                RefusalCode::NotExecutable,
                &resolved,
                format!("{script} is not executable by the portal's user"),
            ));
        }
        Ok(resolved)
    }

    fn list(&self) -> Option<Vec<ScriptEntry>> {
        let mut found: Vec<ScriptEntry> = fs::read_dir(&self.root)
            .ok()?
            .flatten()
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .map(|path| ScriptEntry {
                problem: self.resolve(&path).err(),
                header: ScriptHeader::parse(
                    &fs::read_to_string(self.root.join(&path)).unwrap_or_default(),
                ),
                path,
            })
            .collect();
        found.sort_by(|left, right| left.path.cmp(&right.path));
        Some(found)
    }

    fn editing(&self) -> bool {
        false
    }
}
