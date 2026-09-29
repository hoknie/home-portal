use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use std::sync::Arc;

use portal_model::{ScriptHeader, ScriptPath};

use crate::ports::ProcessIdentity;
use crate::types::{ProblemCode, ScriptFile, ScriptProblem, ScriptTree, Unreadable};

#[derive(Clone)]
pub struct ScriptsDirectory {
    root: PathBuf,
    identity: Arc<dyn ProcessIdentity>,
}

impl ScriptsDirectory {
    pub const LISTED: usize = 500;
    const WRITABLE_BY_OTHERS: u32 = 0o022;
    const OWNER_EXECUTES: u32 = 0o100;
    const GROUP_EXECUTES: u32 = 0o010;
    const OTHERS_EXECUTE: u32 = 0o001;
    const ANYONE_EXECUTES: u32 = 0o111;

    pub fn at(root: PathBuf, identity: Arc<dyn ProcessIdentity>) -> ScriptsDirectory {
        ScriptsDirectory { root, identity }
    }

    pub fn user(&self) -> u32 {
        self.identity.user()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn canonical_root(&self) -> PathBuf {
        fs::canonicalize(&self.root).unwrap_or_else(|_| self.root.clone())
    }

    pub fn resolve(&self, script: &str) -> Result<PathBuf, ScriptProblem> {
        let named = self.root.join(script);
        if let Some((code, problem)) = shape_refusal(script) {
            return Err(ScriptProblem::new(
                code,
                &named,
                format!("the script {problem}"),
            ));
        }
        let root = fs::canonicalize(&self.root).map_err(|_| {
            ScriptProblem::new(
                ProblemCode::NotFound,
                &self.root,
                format!(
                    "the scripts directory {} does not exist",
                    self.root.display()
                ),
            )
        })?;
        let resolved = fs::canonicalize(root.join(script)).map_err(|_| {
            ScriptProblem::new(
                ProblemCode::NotFound,
                &named,
                format!("{script} does not exist in the scripts directory"),
            )
        })?;
        if !resolved.starts_with(&root) || resolved == root {
            return Err(ScriptProblem::new(
                ProblemCode::Outside,
                &named,
                format!("{script} is outside the scripts directory"),
            ));
        }
        let inside = resolved
            .strip_prefix(&root)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some((code, problem)) = shape_refusal(&inside) {
            return Err(ScriptProblem::new(
                code,
                &resolved,
                format!("{script} leads to {inside}, which {problem}"),
            ));
        }
        let metadata = fs::metadata(&resolved).map_err(|error| {
            ScriptProblem::new(
                ProblemCode::NotFound,
                &resolved,
                format!("{script} cannot be read: {error}"),
            )
        })?;
        if !metadata.is_file() {
            return Err(ScriptProblem::new(
                ProblemCode::NotAFile,
                &resolved,
                format!("{script} is not a regular file"),
            ));
        }
        self.check_folders(&root, &resolved, script)?;
        self.check_file(&metadata, &resolved, script)?;
        Ok(resolved)
    }

    pub fn tree(&self) -> Option<ScriptTree> {
        let root = fs::canonicalize(&self.root).ok()?;
        let mut tree = ScriptTree::default();
        self.walk(&root, &root, 1, &mut tree);
        tree.files.sort_by(|left, right| left.path.cmp(&right.path));
        tree.folders.sort();
        Some(tree)
    }

    fn walk(&self, root: &Path, folder: &Path, depth: usize, tree: &mut ScriptTree) {
        let Ok(entries) = fs::read_dir(folder) else {
            return;
        };
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let name = relative.to_string_lossy().to_string();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                if depth < ScriptPath::DEEPEST {
                    tree.folders.push(name);
                    self.walk(root, &path, depth + 1, tree);
                } else {
                    tree.left_out += 1;
                }
                continue;
            }
            if tree.files.len() >= Self::LISTED {
                tree.left_out += 1;
                continue;
            }
            let metadata = fs::symlink_metadata(&path).ok();
            let link = metadata
                .as_ref()
                .is_some_and(|found| found.file_type().is_symlink());
            let size = metadata.as_ref().map_or(0, fs::Metadata::len);
            let unreadable = if link {
                Some(Unreadable::Link)
            } else if size > ScriptFile::LARGEST_TEXT {
                Some(Unreadable::TooLarge)
            } else {
                None
            };
            tree.files.push(ScriptFile {
                problem: self.resolve(&name).err(),
                path: name,
                header: ScriptHeader::default(),
                size,
                modified: metadata.as_ref().and_then(|found| found.modified().ok()),
                mode: metadata
                    .as_ref()
                    .map_or(0, |found| found.permissions().mode() & 0o7777),
                unreadable,
                revision: None,
            });
        }
    }

    fn check_folders(
        &self,
        root: &Path,
        resolved: &Path,
        script: &str,
    ) -> Result<(), ScriptProblem> {
        let mut folder = resolved.parent();
        while let Some(current) = folder {
            let metadata = fs::metadata(current).map_err(|error| {
                ScriptProblem::new(
                    ProblemCode::NotFound,
                    current,
                    format!("{script} cannot be checked: {error}"),
                )
            })?;
            let owner = metadata.uid();
            if owner != self.identity.user() && owner != 0 {
                return Err(ScriptProblem::new(
                    ProblemCode::FolderOwner,
                    current,
                    format!(
                        "{script} is in {}, which is owned by neither the portal's user nor root",
                        current.display()
                    ),
                ));
            }
            if metadata.permissions().mode() & Self::WRITABLE_BY_OTHERS != 0 {
                return Err(ScriptProblem::new(
                    ProblemCode::FolderWritable,
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

    fn check_file(
        &self,
        metadata: &fs::Metadata,
        resolved: &Path,
        script: &str,
    ) -> Result<(), ScriptProblem> {
        let mode = metadata.permissions().mode();
        if mode & Self::WRITABLE_BY_OTHERS != 0 {
            return Err(ScriptProblem::new(
                ProblemCode::Writable,
                resolved,
                format!("{script} can be written by group or others"),
            ));
        }
        let user = self.identity.user();
        if metadata.uid() != user && metadata.uid() != 0 {
            return Err(ScriptProblem::new(
                ProblemCode::Owner,
                resolved,
                format!("{script} is owned by neither the portal's user nor root"),
            ));
        }
        let executable = if user == 0 {
            mode & Self::ANYONE_EXECUTES != 0
        } else if metadata.uid() == user {
            mode & Self::OWNER_EXECUTES != 0
        } else if self.identity.groups().contains(&metadata.gid()) {
            mode & Self::GROUP_EXECUTES != 0
        } else {
            mode & Self::OTHERS_EXECUTE != 0
        };
        if !executable {
            return Err(ScriptProblem::new(
                ProblemCode::NotExecutable,
                resolved,
                format!("{script} is not executable by the portal's user"),
            ));
        }
        Ok(())
    }
}

fn shape_refusal(script: &str) -> Option<(ProblemCode, &'static str)> {
    let problem = ScriptPath::parse(script).err()?;
    Some((ProblemCode::of_shape(problem), problem.message()))
}
