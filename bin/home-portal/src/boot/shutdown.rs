use std::future;

use tokio::signal;

use crate::types::{Ended, Restart};

pub async fn requested(restart: Restart) -> Ended {
    let ended = tokio::select! {
        _ = interrupted() => Ended::Stopped,
        _ = terminated() => Ended::Stopped,
        _ = restart.wanted() => Ended::Restart,
    };
    match ended {
        Ended::Stopped => tracing::info!("shutdown requested"),
        Ended::Restart => tracing::info!("restart requested"),
    }
    ended
}

async fn interrupted() {
    if signal::ctrl_c().await.is_err() {
        future::pending::<()>().await;
    }
}

#[cfg(unix)]
async fn terminated() {
    match signal::unix::signal(signal::unix::SignalKind::terminate()) {
        Ok(mut stream) => {
            stream.recv().await;
        }
        Err(_) => future::pending::<()>().await,
    }
}

#[cfg(not(unix))]
async fn terminated() {
    future::pending::<()>().await;
}
