use std::future;

use tokio::signal;

pub async fn requested() {
    tokio::select! {
        _ = interrupted() => {},
        _ = terminated() => {},
    }
    tracing::info!("shutdown requested");
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
