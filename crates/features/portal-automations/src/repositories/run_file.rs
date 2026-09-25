use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::types::{RunRecord, StoredRun};

pub struct RunFile {
    path: PathBuf,
    compacted: Mutex<u64>,
}

impl RunFile {
    pub const FILE: &'static str = "runs.ndjson";
    pub const TEMPORARY_SUFFIX: &'static str = ".tmp";
    pub const PRIVATE_MODE: u32 = 0o600;
    pub const DIRECTORY_MODE: u32 = 0o700;
    pub const COMPACT_AFTER_BYTES: u64 = 1024 * 1024;

    pub fn at(directory: &Path) -> RunFile {
        RunFile {
            path: directory.join(Self::FILE),
            compacted: Mutex::new(0),
        }
    }

    pub fn load(&self, kept: usize) -> (Vec<RunRecord>, u64) {
        let _ = fs::remove_file(self.temporary());
        let Ok(file) = fs::File::open(&self.path) else {
            return (Vec::new(), 0);
        };
        let mut reader = BufReader::new(file);
        let mut order: Vec<u64> = Vec::new();
        let mut latest: HashMap<u64, StoredRun> = HashMap::new();
        let mut broken = 0usize;
        let mut highest = 0u64;
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            match reader.read_until(b'\n', &mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if buffer.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            match serde_json::from_slice::<StoredRun>(&buffer) {
                Ok(run) => {
                    highest = highest.max(run.id);
                    if !latest.contains_key(&run.id) {
                        order.push(run.id);
                    }
                    latest.insert(run.id, run);
                }
                Err(_) => {
                    broken += 1;
                    highest = highest.max(Self::id_in(&buffer));
                }
            }
        }
        if broken > 0 {
            tracing::warn!(path = %self.path.display(), broken, "some lines of the run journal could not be read and were skipped");
        }
        let records = order
            .iter()
            .rev()
            .filter_map(|id| latest.remove(id))
            .filter_map(StoredRun::into_record)
            .take(kept)
            .collect();
        (records, highest)
    }

    fn id_in(line: &[u8]) -> u64 {
        const PREFIX: &[u8] = b"{\"id\":";
        line.strip_prefix(PREFIX)
            .map(|rest| {
                rest.iter()
                    .take_while(|byte| byte.is_ascii_digit())
                    .fold(0u64, |id, digit| {
                        id.saturating_mul(10)
                            .saturating_add(u64::from(digit - b'0'))
                    })
            })
            .unwrap_or(0)
    }

    pub fn append(&self, records: &[RunRecord]) -> io::Result<()> {
        if records.is_empty() {
            return Ok(());
        }
        Self::create_private_directory(self.path.parent().unwrap_or(Path::new(".")))?;
        let mut file = Self::open(&self.path, false)?;
        file.write_all(Self::text_of(records)?.as_bytes())?;
        file.sync_data()?;
        Ok(())
    }

    pub fn needs_compacting(&self) -> bool {
        let compacted = *self
            .compacted
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let limit = Self::COMPACT_AFTER_BYTES.max(compacted.saturating_mul(2));
        fs::metadata(&self.path).is_ok_and(|metadata| metadata.len() > limit)
    }

    pub fn rewrite(&self, newest_first: &[RunRecord]) -> io::Result<()> {
        Self::create_private_directory(self.path.parent().unwrap_or(Path::new(".")))?;
        let temporary = self.temporary();
        let oldest_first: Vec<RunRecord> = newest_first.iter().rev().cloned().collect();
        let text = Self::text_of(&oldest_first)?;
        let mut file = Self::open(&temporary, true)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        if let Some(Ok(directory)) = self.path.parent().map(fs::File::open) {
            let _ = directory.sync_all();
        }
        *self
            .compacted
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = text.len() as u64;
        Ok(())
    }

    fn temporary(&self) -> PathBuf {
        let mut temporary = self.path.clone().into_os_string();
        temporary.push(Self::TEMPORARY_SUFFIX);
        PathBuf::from(temporary)
    }

    fn text_of(records: &[RunRecord]) -> io::Result<String> {
        let mut text = String::new();
        for record in records {
            text.push_str(
                &serde_json::to_string(&StoredRun::of(record)).map_err(io::Error::other)?,
            );
            text.push('\n');
        }
        Ok(text)
    }

    #[cfg(unix)]
    fn open(path: &Path, truncate: bool) -> io::Result<fs::File> {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let file = OpenOptions::new()
            .create(true)
            .append(!truncate)
            .write(true)
            .truncate(truncate)
            .mode(Self::PRIVATE_MODE)
            .open(path)?;
        file.set_permissions(fs::Permissions::from_mode(Self::PRIVATE_MODE))?;
        Ok(file)
    }

    #[cfg(not(unix))]
    fn open(path: &Path, truncate: bool) -> io::Result<fs::File> {
        OpenOptions::new()
            .create(true)
            .append(!truncate)
            .write(true)
            .truncate(truncate)
            .open(path)
    }

    #[cfg(not(unix))]
    fn create_private_directory(directory: &Path) -> io::Result<()> {
        fs::create_dir_all(directory)
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
}
