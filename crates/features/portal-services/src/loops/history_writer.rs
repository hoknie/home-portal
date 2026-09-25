use std::sync::Arc;
use std::time::Duration;

use crate::repositories::HistoryFiles;
use crate::services::StatusBoard;

pub struct HistoryWriter {
    pub board: Arc<StatusBoard>,
    pub files: Arc<HistoryFiles>,
}

impl HistoryWriter {
    pub const PERIOD: Duration = Duration::from_secs(300);

    pub async fn run(self: Arc<Self>) {
        loop {
            tokio::time::sleep(Self::PERIOD).await;
            self.flush();
        }
    }

    pub fn flush(&self) {
        for (id, history) in self.board.take_dirty() {
            if let Err(error) = self.files.save(&id, &history) {
                tracing::warn!(service = %id, %error, "cannot write the service history");
            }
        }
        for id in self.board.take_forgotten() {
            if let Err(error) = self.files.delete(&id) {
                tracing::warn!(service = %id, %error, "cannot delete the service history");
            }
        }
    }
}
