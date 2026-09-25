use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use crate::repositories::RunFile;
use crate::services::Journal;

pub struct JournalWriter {
    pub journal: Arc<Journal>,
    pub file: Arc<RunFile>,
    pub writing: Mutex<bool>,
}

impl JournalWriter {
    pub const PERIOD: Duration = Duration::from_secs(1);

    pub fn new(journal: Arc<Journal>, file: Arc<RunFile>) -> JournalWriter {
        JournalWriter {
            journal,
            file,
            writing: Mutex::new(false),
        }
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            tokio::time::sleep(Self::PERIOD).await;
            let writer = self.clone();
            let _ = tokio::task::spawn_blocking(move || writer.flush()).await;
        }
    }

    pub fn flush(&self) {
        let mut rewrite = self.writing.lock().unwrap_or_else(PoisonError::into_inner);
        let records = self.journal.take_unwritten();
        let written = if *rewrite || self.file.needs_compacting() {
            self.file.rewrite(&self.journal.snapshot())
        } else {
            self.file.append(&records)
        };
        *rewrite = match written {
            Ok(()) => false,
            Err(error) => {
                tracing::warn!(%error, "cannot write the run journal; it will be rewritten whole next time");
                true
            }
        };
    }
}
