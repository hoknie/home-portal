use std::path::{Path, PathBuf};
use std::time::Duration;

use super::board::FailureBoard;
use super::fingerprint::Fingerprint;
use super::main_file::MainFile;
use super::problems::problems_of;
use crate::boot::prepare;
use crate::types::Restart;

pub const RECHECK_EVERY: Duration = Duration::from_secs(2);

pub async fn recheck_forever(main: PathBuf, board: FailureBoard, restart: Restart) {
    let mut seen = Fingerprint::of(&main);
    loop {
        tokio::time::sleep(RECHECK_EVERY).await;
        let now = Fingerprint::of(&main);
        if now == seen {
            continue;
        }
        seen = now;
        if recheck(&main, &board).await {
            restart.request();
            return;
        }
    }
}

pub async fn recheck(main: &Path, board: &FailureBoard) -> bool {
    let outcome = tokio::task::spawn_blocking(prepare).await;
    let error = match outcome {
        Ok(Ok(_)) => {
            tracing::info!("the configuration is valid now; starting afresh");
            return true;
        }
        Ok(Err(error)) => error,
        Err(panic) => {
            tracing::error!(%panic, "checking the configuration panicked");
            return false;
        }
    };
    let address = board.current().main.address;
    let read = MainFile {
        address,
        ..MainFile::read(main)
    };
    if board.checked(problems_of(&error), read) {
        tracing::error!("the portal still cannot start: {error}");
    }
    false
}
