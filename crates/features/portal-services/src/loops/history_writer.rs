use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use crate::repositories::HistoryFiles;
use time::OffsetDateTime;

use crate::services::{StatusBoard, persist};

pub struct HistoryWriter {
    pub board: Arc<StatusBoard>,
    pub files: Arc<HistoryFiles>,
    pub writing: Mutex<()>,
}

impl HistoryWriter {
    pub const PERIOD: Duration = Duration::from_secs(300);

    pub fn new(board: Arc<StatusBoard>, files: Arc<HistoryFiles>) -> HistoryWriter {
        HistoryWriter {
            board,
            files,
            writing: Mutex::new(()),
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
        let _writing = self.writing.lock().unwrap_or_else(PoisonError::into_inner);
        let forgotten = self.board.take_forgotten();
        for (id, write) in self.board.take_dirty() {
            let now = OffsetDateTime::now_utc().unix_timestamp();
            if let Err(error) = persist(&self.files, &id, &write, now) {
                tracing::warn!(service = %id, %error, "cannot write the service history; it will be rewritten whole next time");
                self.board.rewrite_later(&id);
            }
        }
        for id in forgotten {
            if let Err(error) = self.files.delete(&id) {
                tracing::warn!(service = %id, %error, "cannot delete the service history");
            }
        }
    }
}
