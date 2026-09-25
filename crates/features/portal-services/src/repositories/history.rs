use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::services::ServiceHistory;

pub struct HistoryFiles {
    directory: PathBuf,
}

impl HistoryFiles {
    pub const DIRECTORY: &'static str = "history";
    pub const EXTENSION: &'static str = "json";
    pub const BROKEN_SUFFIX: &'static str = ".broken";
    pub const TEMPORARY_SUFFIX: &'static str = ".tmp";
    pub const PRIVATE_MODE: u32 = 0o600;
    pub const DIRECTORY_MODE: u32 = 0o700;

    pub fn beside(configuration: &Path) -> HistoryFiles {
        let parent = configuration.parent().unwrap_or(Path::new("."));
        HistoryFiles {
            directory: parent.join(Self::DIRECTORY),
        }
    }

    pub fn file_of(&self, id: &str) -> PathBuf {
        self.directory.join(format!("{id}.{}", Self::EXTENSION))
    }

    pub fn load(&self, wanted: &[String]) -> HashMap<String, ServiceHistory> {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return HashMap::new();
        };
        let mut loaded = HashMap::new();
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.extension().and_then(|extension| extension.to_str()) != Some(Self::EXTENSION) {
                continue;
            }
            let Some(id) = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            if !wanted.contains(&id) {
                if let Err(error) = fs::remove_file(&path) {
                    tracing::warn!(path = %path.display(), %error, "cannot delete the history of a removed service");
                }
                continue;
            }
            match Self::read(&path) {
                Ok(history) => {
                    loaded.insert(id, history);
                }
                Err(error) => self.quarantine(&path, &error),
            }
        }
        loaded
    }

    pub fn save(&self, id: &str, history: &ServiceHistory) -> io::Result<()> {
        Self::create_private_directory(&self.directory)?;
        let path = self.file_of(id);
        let mut temporary = path.clone().into_os_string();
        temporary.push(Self::TEMPORARY_SUFFIX);
        let temporary = PathBuf::from(temporary);
        let text = serde_json::to_string(history).map_err(io::Error::other)?;
        let mut file = Self::open_private(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, &path)
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        match fs::remove_file(self.file_of(id)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn read(path: &Path) -> io::Result<ServiceHistory> {
        let text = fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(io::Error::other)
    }

    fn quarantine(&self, path: &Path, error: &io::Error) {
        let mut broken = path.as_os_str().to_os_string();
        broken.push(Self::BROKEN_SUFFIX);
        tracing::warn!(path = %path.display(), %error, "a history file cannot be read; it is set aside and the history starts empty");
        if let Err(error) = fs::rename(path, &broken) {
            tracing::warn!(path = %path.display(), %error, "cannot set the history file aside");
        }
    }

    #[cfg(unix)]
    fn create_private_directory(directory: &Path) -> io::Result<()> {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        if directory.is_dir() {
            return Ok(());
        }
        fs::DirBuilder::new()
            .recursive(true)
            .mode(Self::DIRECTORY_MODE)
            .create(directory)?;
        fs::set_permissions(directory, fs::Permissions::from_mode(Self::DIRECTORY_MODE))
    }

    #[cfg(not(unix))]
    fn create_private_directory(directory: &Path) -> io::Result<()> {
        fs::create_dir_all(directory)
    }

    #[cfg(unix)]
    fn open_private(path: &Path) -> io::Result<fs::File> {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(Self::PRIVATE_MODE)
            .open(path)?;
        file.set_permissions(fs::Permissions::from_mode(Self::PRIVATE_MODE))?;
        Ok(file)
    }

    #[cfg(not(unix))]
    fn open_private(path: &Path) -> io::Result<fs::File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
    }
}
