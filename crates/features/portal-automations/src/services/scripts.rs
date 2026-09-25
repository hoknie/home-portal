use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::clients::{effective_groups, effective_user};
use crate::helpers::{DEEPEST, script_shape};
use crate::types::{Refusal, RefusalCode, ScriptEntry};

#[derive(Debug, Clone)]
pub struct ScriptsDirectory {
    root: PathBuf,
}

impl ScriptsDirectory {
    pub const LISTED: usize = 500;
    const WRITABLE_BY_OTHERS: u32 = 0o022;
    const OWNER_EXECUTES: u32 = 0o100;
    const GROUP_EXECUTES: u32 = 0o010;
    const OTHERS_EXECUTE: u32 = 0o001;
    const ANYONE_EXECUTES: u32 = 0o111;

    pub fn at(root: PathBuf) -> ScriptsDirectory {
        ScriptsDirectory { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn canonical_root(&self) -> PathBuf {
        fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone())
    }

    pub fn resolve(&self, script: &str) -> Result<PathBuf, Refusal> {
        let named = self.root.join(script);
        if let Some((code, problem)) = script_shape(script) {
            return Err(Refusal::new(code, &named, format!("the script {problem}")));
        }
        let root = fs::canonicalize(&self.root).map_err(|_| {
            Refusal::new(
                RefusalCode::NotFound,
                &self.root,
                format!(
                    "the scripts directory {} does not exist",
                    self.root.display()
                ),
            )
        })?;
        let resolved = fs::canonicalize(root.join(script)).map_err(|_| {
            Refusal::new(
                RefusalCode::NotFound,
                &named,
                format!("{script} does not exist in the scripts directory"),
            )
        })?;
        if !resolved.starts_with(&root) || resolved == root {
            return Err(Refusal::new(
                RefusalCode::Outside,
                &named,
                format!("{script} is outside the scripts directory"),
            ));
        }
        let inside = resolved
            .strip_prefix(&root)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some((code, problem)) = script_shape(&inside) {
            return Err(Refusal::new(
                code,
                &resolved,
                format!("{script} leads to {inside}, which {problem}"),
            ));
        }
        let metadata = fs::metadata(&resolved).map_err(|error| {
            Refusal::new(
                RefusalCode::NotFound,
                &resolved,
                format!("{script} cannot be read: {error}"),
            )
        })?;
        if !metadata.is_file() {
            return Err(Refusal::new(
                RefusalCode::NotAFile,
                &resolved,
                format!("{script} is not a regular file"),
            ));
        }
        Self::check_folders(&root, &resolved, script)?;
        Self::check_file(&metadata, &resolved, script)?;
        Ok(resolved)
    }

    pub fn list(&self) -> Option<Vec<ScriptEntry>> {
        let root = fs::canonicalize(&self.root).ok()?;
        let mut found = Vec::new();
        self.walk(&root, &root, 1, &mut found);
        found.sort_by(|left, right| left.path.cmp(&right.path));
        Some(found)
    }

    fn walk(&self, root: &Path, folder: &Path, depth: usize, found: &mut Vec<ScriptEntry>) {
        let Ok(entries) = fs::read_dir(folder) else {
            return;
        };
        for entry in entries.flatten() {
            if found.len() >= Self::LISTED {
                return;
            }
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let name = relative.to_string_lossy().to_string();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                if depth < DEEPEST {
                    self.walk(root, &path, depth + 1, found);
                }
                continue;
            }
            let problem = self.resolve(&name).err();
            found.push(ScriptEntry {
                path: name,
                problem,
            });
        }
    }

    fn check_folders(root: &Path, resolved: &Path, script: &str) -> Result<(), Refusal> {
        let mut folder = resolved.parent();
        while let Some(current) = folder {
            let metadata = fs::metadata(current).map_err(|error| {
                Refusal::new(
                    RefusalCode::NotFound,
                    current,
                    format!("{script} cannot be checked: {error}"),
                )
            })?;
            let owner = metadata.uid();
            if owner != effective_user() && owner != 0 {
                return Err(Refusal::new(
                    RefusalCode::FolderOwner,
                    current,
                    format!(
                        "{script} is in {}, which is owned by neither the portal's user nor root",
                        current.display()
                    ),
                ));
            }
            if metadata.permissions().mode() & Self::WRITABLE_BY_OTHERS != 0 {
                return Err(Refusal::new(
                    RefusalCode::FolderWritable,
                    current,
                    format!(
                        "{script} is in {}, which group or others can write",
                        current.display()
                    ),
                ));
            }
            if current == root {
                return Ok(());
            }
            folder = current.parent();
        }
        Ok(())
    }

    fn check_file(metadata: &fs::Metadata, resolved: &Path, script: &str) -> Result<(), Refusal> {
        let mode = metadata.permissions().mode();
        if mode & Self::WRITABLE_BY_OTHERS != 0 {
            return Err(Refusal::new(
                RefusalCode::Writable,
                resolved,
                format!("{script} can be written by group or others"),
            ));
        }
        let user = effective_user();
        if metadata.uid() != user && metadata.uid() != 0 {
            return Err(Refusal::new(
                RefusalCode::Owner,
                resolved,
                format!("{script} is owned by neither the portal's user nor root"),
            ));
        }
        let executable = if user == 0 {
            mode & Self::ANYONE_EXECUTES != 0
        } else if metadata.uid() == user {
            mode & Self::OWNER_EXECUTES != 0
        } else if effective_groups().contains(&metadata.gid()) {
            mode & Self::GROUP_EXECUTES != 0
        } else {
            mode & Self::OTHERS_EXECUTE != 0
        };
        if !executable {
            return Err(Refusal::new(
                RefusalCode::NotExecutable,
                resolved,
                format!("{script} is not executable by the portal's user"),
            ));
        }
        Ok(())
    }
}
