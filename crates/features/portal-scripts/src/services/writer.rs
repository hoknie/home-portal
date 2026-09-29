use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use portal_config::Revision;
use portal_feature::ApiError;
use portal_model::ScriptPath;

use crate::types::ScriptFile;

static WRITES: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct ScriptWriter {
    root: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl ScriptWriter {
    pub const MODE: u32 = 0o700;
    pub const MOST_FILES: usize = 500;
    pub const STALE: Duration = Duration::from_secs(60 * 60);
    const TEMPORARY: &'static str = ".tmp-";

    pub fn at(root: PathBuf) -> ScriptWriter {
        ScriptWriter {
            root,
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn read(&self, path: &ScriptPath) -> Result<(String, Revision), ApiError> {
        let target = self.existing(path)?;
        let bytes = fs::read(&target).map_err(|error| ApiError::Internal(error.to_string()))?;
        if bytes.len() as u64 > ScriptFile::LARGEST_TEXT {
            return Err(ApiError::invalid(
                "path",
                "is larger than 256 KiB and cannot be edited here",
            ));
        }
        let revision = Revision::of(&bytes);
        let text = String::from_utf8(bytes).map_err(|_| {
            ApiError::invalid("path", "is not UTF-8 text and cannot be edited here")
        })?;
        if text.contains('\0') {
            return Err(ApiError::invalid(
                "path",
                "is not text and cannot be edited here",
            ));
        }
        Ok((text, revision))
    }

    pub fn create(&self, path: &ScriptPath, content: &str) -> Result<Revision, ApiError> {
        let _held = self.hold();
        Self::check_content(content)?;
        let target = self.place(path)?;
        if fs::symlink_metadata(&target).is_ok() {
            return Err(ApiError::Conflict(format!(
                "{} already exists",
                path.text()
            )));
        }
        if self.count() >= Self::MOST_FILES {
            return Err(ApiError::invalid(
                "path",
                "cannot be created; the scripts directory already holds 500 files",
            ));
        }
        self.write_whole(&target, content)
    }

    pub fn replace(
        &self,
        path: &ScriptPath,
        content: &str,
        expected: &Revision,
    ) -> Result<Revision, ApiError> {
        let _held = self.hold();
        Self::check_content(content)?;
        let target = self.existing(path)?;
        self.check_revision(&target, expected)?;
        self.write_whole(&target, content)
    }

    pub fn delete(&self, path: &ScriptPath, expected: &Revision) -> Result<(), ApiError> {
        let _held = self.hold();
        let target = self.existing(path)?;
        self.check_revision(&target, expected)?;
        fs::remove_file(&target).map_err(|error| ApiError::Internal(error.to_string()))?;
        Self::sync_folder(&target);
        Ok(())
    }

    pub fn rename(
        &self,
        from: &ScriptPath,
        to: &ScriptPath,
        expected: &Revision,
    ) -> Result<Revision, ApiError> {
        let _held = self.hold();
        let source = self.existing(from)?;
        self.check_revision(&source, expected)?;
        let target = self.place(to)?;
        if fs::symlink_metadata(&target).is_ok() {
            return Err(ApiError::Conflict(format!("{} already exists", to.text())));
        }
        fs::rename(&source, &target).map_err(|error| ApiError::Internal(error.to_string()))?;
        Self::sync_folder(&source);
        Self::sync_folder(&target);
        Ok(expected.clone())
    }

    pub fn create_folder(&self, name: Option<&str>) -> Result<(), ApiError> {
        let _held = self.hold();
        let mut builder = DirBuilder::new();
        builder.mode(Self::MODE);
        if fs::symlink_metadata(&self.root).is_err() {
            builder
                .create(&self.root)
                .map_err(|error| ApiError::Internal(format!("{}: {error}", self.root.display())))?;
        }
        let Some(name) = name else {
            return Ok(());
        };
        let folder = self.root()?.join(name);
        if fs::symlink_metadata(&folder).is_ok() {
            return Err(ApiError::Conflict(format!("{name} already exists")));
        }
        builder
            .create(&folder)
            .map_err(|error| ApiError::Internal(error.to_string()))
    }

    pub fn delete_folder(&self, name: &str) -> Result<(), ApiError> {
        let _held = self.hold();
        let folder = self.root()?.join(name);
        let metadata =
            fs::symlink_metadata(&folder).map_err(|_| ApiError::NotFound("no such folder"))?;
        if metadata.file_type().is_symlink() {
            return Err(ApiError::invalid("name", "is a symbolic link"));
        }
        if !metadata.is_dir() {
            return Err(ApiError::NotFound("no such folder"));
        }
        let mut entries: Vec<String> = fs::read_dir(&folder)
            .map_err(|error| ApiError::Internal(error.to_string()))?
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        if !entries.is_empty() {
            return Err(ApiError::Conflict(format!(
                "{name} is not empty: {}",
                entries.join(", ")
            )));
        }
        fs::remove_dir(&folder).map_err(|error| ApiError::Internal(error.to_string()))
    }

    pub fn sweep(&self) -> usize {
        let Ok(root) = self.root() else {
            return 0;
        };
        let mut folders = vec![root.clone()];
        if let Ok(entries) = fs::read_dir(&root) {
            folders.extend(
                entries
                    .flatten()
                    .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                    .map(|entry| entry.path()),
            );
        }
        let now = SystemTime::now();
        let mut removed = 0;
        for folder in folders {
            let Ok(entries) = fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let stale = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .is_ok_and(|modified| {
                        now.duration_since(modified).unwrap_or_default() > Self::STALE
                    });
                if name.starts_with('.')
                    && name.contains(Self::TEMPORARY)
                    && stale
                    && fs::remove_file(entry.path()).is_ok()
                {
                    removed += 1;
                }
            }
        }
        removed
    }

    fn hold(&self) -> std::sync::MutexGuard<'_, ()> {
        self.lock.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn root(&self) -> Result<PathBuf, ApiError> {
        fs::canonicalize(&self.root)
            .map_err(|_| ApiError::NotFound("the scripts directory does not exist"))
    }

    fn place(&self, path: &ScriptPath) -> Result<PathBuf, ApiError> {
        let root = self.root()?;
        let folder = match path.folder() {
            Some(folder) => {
                let place = root.join(folder);
                let metadata = fs::symlink_metadata(&place)
                    .map_err(|_| ApiError::invalid("path", "names a folder that does not exist"))?;
                if metadata.file_type().is_symlink() {
                    return Err(ApiError::invalid("path", "passes through a symbolic link"));
                }
                if !metadata.is_dir() {
                    return Err(ApiError::invalid(
                        "path",
                        "names a folder that is not a folder",
                    ));
                }
                place
            }
            None => root,
        };
        let target = folder.join(path.name());
        if fs::symlink_metadata(&target).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(ApiError::invalid("path", "is a symbolic link"));
        }
        Ok(target)
    }

    fn existing(&self, path: &ScriptPath) -> Result<PathBuf, ApiError> {
        let target = self.place(path)?;
        match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.is_file() => Ok(target),
            _ => Err(ApiError::NotFound("no such script")),
        }
    }

    fn check_content(content: &str) -> Result<(), ApiError> {
        if content.len() as u64 > ScriptFile::LARGEST_TEXT {
            return Err(ApiError::invalid("content", "is larger than 256 KiB"));
        }
        if content.contains('\0') {
            return Err(ApiError::invalid("content", "must be text"));
        }
        Ok(())
    }

    fn check_revision(&self, target: &Path, expected: &Revision) -> Result<(), ApiError> {
        let bytes = fs::read(target).map_err(|error| ApiError::Internal(error.to_string()))?;
        if Revision::of(&bytes) != *expected {
            return Err(ApiError::Conflict(
                "the script changed since it was read; reload it".to_string(),
            ));
        }
        Ok(())
    }

    fn count(&self) -> usize {
        let Ok(root) = self.root() else {
            return 0;
        };
        let visible = |path: &Path| {
            fs::read_dir(path)
                .into_iter()
                .flatten()
                .flatten()
                .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        };
        visible(&root)
            .map(|entry| {
                if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    visible(&entry.path())
                        .filter(|inner| !inner.file_type().is_ok_and(|kind| kind.is_dir()))
                        .count()
                } else {
                    1
                }
            })
            .sum()
    }

    fn write_whole(&self, target: &Path, content: &str) -> Result<Revision, ApiError> {
        let folder = target.parent().unwrap_or(&self.root);
        let name = target
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = folder.join(format!(
            ".{name}{}{}-{nanos}-{}",
            Self::TEMPORARY,
            std::process::id(),
            WRITES.fetch_add(1, Ordering::Relaxed)
        ));
        let written = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(Self::MODE)
            .open(&temporary)
            .and_then(|mut file| {
                file.write_all(content.as_bytes())?;
                file.sync_all()
            })
            .and_then(|()| fs::rename(&temporary, target));
        if let Err(error) = written {
            let _ = fs::remove_file(&temporary);
            return Err(ApiError::Internal(format!("{}: {error}", target.display())));
        }
        Self::sync_folder(target);
        Ok(Revision::of(content.as_bytes()))
    }

    fn sync_folder(path: &Path) {
        if let Some(folder) = path.parent()
            && let Ok(handle) = File::open(folder)
        {
            let _ = handle.sync_all();
        }
    }
}
