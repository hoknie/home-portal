use std::collections::{BTreeSet, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::types::HistoryLine;

pub struct HistoryFiles {
    directory: PathBuf,
    compacted: Mutex<HashMap<String, u64>>,
}

impl HistoryFiles {
    pub const EXTENSION: &'static str = "ndjson";
    pub const BROKEN_SUFFIX: &'static str = ".broken";
    pub const TEMPORARY_SUFFIX: &'static str = ".tmp";
    pub const PRIVATE_MODE: u32 = 0o600;
    pub const DIRECTORY_MODE: u32 = 0o700;
    pub const COMPACT_AFTER_BYTES: u64 = 1024 * 1024;

    pub fn at(directory: PathBuf) -> HistoryFiles {
        HistoryFiles {
            directory,
            compacted: Mutex::new(HashMap::new()),
        }
    }

    pub fn file_of(&self, id: &str) -> PathBuf {
        self.directory.join(format!("{id}.{}", Self::EXTENSION))
    }

    pub fn ids(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let mut ids = BTreeSet::new();
        for path in entries.flatten().map(|entry| entry.path()) {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if name.ends_with(Self::TEMPORARY_SUFFIX) {
                let _ = fs::remove_file(&path);
                continue;
            }
            let extension = path.extension().and_then(|extension| extension.to_str());
            if extension != Some(Self::EXTENSION) {
                continue;
            }
            if let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) {
                ids.insert(id.to_string());
            }
        }
        ids.into_iter().collect()
    }

    pub fn recover(&self, id: &str) -> Option<Vec<HistoryLine>> {
        let lines_path = self.file_of(id);
        let mut lines = Vec::new();
        if lines_path.exists() {
            match Self::read_lines(&lines_path) {
                Ok(read) => lines.extend(read),
                Err(error) => self.quarantine(&lines_path, &error),
            }
        }
        if lines.is_empty() && !lines_path.exists() {
            return None;
        }
        Some(lines)
    }

    pub fn read(&self, id: &str) -> io::Result<Vec<HistoryLine>> {
        Self::read_lines(&self.file_of(id))
    }

    pub fn append(&self, id: &str, lines: &[HistoryLine]) -> io::Result<()> {
        Self::create_private_directory(&self.directory)?;
        let mut file = Self::open_appending(&self.file_of(id))?;
        file.write_all(Self::text_of(lines)?.as_bytes())?;
        file.sync_data()
    }

    pub fn needs_compacting(&self, id: &str) -> bool {
        fs::metadata(self.file_of(id)).is_ok_and(|metadata| metadata.len() > self.limit_of(id))
    }

    fn limit_of(&self, id: &str) -> u64 {
        let compacted = self
            .compacted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .copied()
            .unwrap_or(0);
        Self::COMPACT_AFTER_BYTES.max(compacted.saturating_mul(2))
    }

    pub fn rewrite(&self, id: &str, lines: &[HistoryLine]) -> io::Result<()> {
        Self::create_private_directory(&self.directory)?;
        let path = self.file_of(id);
        let mut temporary = path.clone().into_os_string();
        temporary.push(Self::TEMPORARY_SUFFIX);
        let temporary = PathBuf::from(temporary);
        let text = Self::text_of(lines)?;
        let mut file = Self::open_private(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, &path)?;
        if let Ok(directory) = fs::File::open(&self.directory) {
            let _ = directory.sync_all();
        }
        self.compacted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_string(), text.len() as u64);
        Ok(())
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        match fs::remove_file(self.file_of(id)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn text_of(lines: &[HistoryLine]) -> io::Result<String> {
        let mut text = String::new();
        for line in lines {
            text.push_str(&serde_json::to_string(line).map_err(io::Error::other)?);
            text.push('\n');
        }
        Ok(text)
    }

    fn read_lines(path: &Path) -> io::Result<Vec<HistoryLine>> {
        let mut reader = BufReader::new(fs::File::open(path)?);
        let mut lines = Vec::new();
        let mut broken = 0usize;
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            if reader.read_until(b'\n', &mut buffer)? == 0 {
                break;
            }
            if buffer.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            match serde_json::from_slice::<HistoryLine>(&buffer) {
                Ok(line) => lines.push(line),
                Err(_) => broken += 1,
            }
        }
        if lines.is_empty() && broken > 0 {
            return Err(io::Error::other("no line of the history could be read"));
        }
        if broken > 0 {
            tracing::warn!(path = %path.display(), broken, "some lines of a history file could not be read and were skipped");
        }
        Ok(lines)
    }

    fn quarantine(&self, path: &Path, error: &io::Error) {
        let mut broken = path.as_os_str().to_os_string();
        broken.push(Self::BROKEN_SUFFIX);
        tracing::warn!(path = %path.display(), %error, "a history file cannot be read; it is set aside");
        if let Err(error) = fs::rename(path, &broken) {
            tracing::warn!(path = %path.display(), %error, "cannot set the history file aside");
        }
    }

    #[cfg(unix)]
    fn open_appending(path: &Path) -> io::Result<fs::File> {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(Self::PRIVATE_MODE)
            .open(path)?;
        file.set_permissions(fs::Permissions::from_mode(Self::PRIVATE_MODE))?;
        Ok(file)
    }

    #[cfg(not(unix))]
    fn open_appending(path: &Path) -> io::Result<fs::File> {
        OpenOptions::new().create(true).append(true).open(path)
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
