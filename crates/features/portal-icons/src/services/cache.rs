use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use time::OffsetDateTime;

use crate::helpers::{digest_of, extension_of};
use crate::types::{Entry, StoredIcon};

pub struct IconCache {
    directory: PathBuf,
    entries: Mutex<BTreeMap<String, Entry>>,
}

impl IconCache {
    pub const INDEX: &'static str = "index.json";
    pub const LIFETIME_DAYS: i64 = 7;

    pub fn open(directory: PathBuf) -> IconCache {
        let _ = fs::create_dir_all(&directory);
        let entries = fs::read_to_string(directory.join(Self::INDEX))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        IconCache {
            directory,
            entries: Mutex::new(entries),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn entry(&self, service: &str) -> Option<Entry> {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(service)
            .cloned()
    }

    pub fn stale(&self, service: &str, source: &str, now: OffsetDateTime) -> bool {
        match self.entry(service) {
            None => true,
            Some(entry) => {
                entry.source != source
                    || (now - entry.fetched_at) > time::Duration::days(Self::LIFETIME_DAYS)
                    || !self.path_of(&entry).is_file()
            }
        }
    }

    pub fn read(&self, service: &str) -> Option<StoredIcon> {
        let entry = self.entry(service)?;
        let bytes = fs::read(self.path_of(&entry)).ok()?;
        Some(StoredIcon {
            bytes,
            content_type: entry.content_type,
            digest: entry.digest,
        })
    }

    pub fn store(
        &self,
        service: &str,
        source: &str,
        bytes: &[u8],
        content_type: &str,
        now: OffsetDateTime,
    ) -> Result<(), String> {
        let entry = Entry {
            source: source.to_string(),
            digest: digest_of(&format!("{source}:{}", bytes.len())),
            content_type: content_type.to_string(),
            fetched_at: now,
        };
        fs::write(self.path_of(&entry), bytes).map_err(|error| error.to_string())?;
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries.insert(service.to_string(), entry);
        let text = serde_json::to_string_pretty(&*entries).map_err(|error| error.to_string())?;
        fs::write(self.directory.join(Self::INDEX), text).map_err(|error| error.to_string())
    }

    fn path_of(&self, entry: &Entry) -> PathBuf {
        self.directory.join(format!(
            "{}.{}",
            entry.digest,
            extension_of(&entry.content_type)
        ))
    }
}
