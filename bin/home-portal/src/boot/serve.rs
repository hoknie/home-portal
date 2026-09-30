use std::net::SocketAddr;
use std::pin::pin;
use std::time::Duration;

use axum::extract::ConnectInfo;
use axum::{Extension, Router};
use hyper::server::conn::http1;
use hyper_util::rt::{TokioIo, TokioTimer};
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinSet;

use super::{lifecycle, shutdown};
use crate::types::{BootError, Ended, Registry, Signals};

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub header_read: Duration,
    pub drain: Duration,
}

impl Limits {
    pub const SERVED: Limits = Limits {
        header_read: Duration::from_secs(30),
        drain: Duration::from_secs(15),
    };
}

pub async fn serve(
    listener: TcpListener,
    router: Router,
    registry: &Registry,
    signals: Signals,
) -> Result<Ended, BootError> {
    let address = listener.local_addr().map_err(|source| BootError::Serve {
        address: SocketAddr::from(([0, 0, 0, 0], 0)),
        source,
    })?;
    tracing::info!(%address, "listening");
    let requested = shutdown::requested(signals, registry.restart.clone());
    let served = accept_until(listener, router, requested, Limits::SERVED).await;
    lifecycle::stopping(registry, address).await;
    for feature in &registry.features {
        feature.stop();
    }
    tracing::info!("stopped");
    served.map_err(|source| BootError::Serve { address, source })?;
    Ok(if registry.restart.requested() {
        Ended::Restart
    } else {
        Ended::Stopped
    })
}

pub async fn accept_until(
    listener: TcpListener,
    router: Router,
    stop: impl Future<Output = Ended>,
    limits: Limits,
) -> std::io::Result<()> {
    let (closing, closed) = watch::channel(false);
    let mut connections = JoinSet::new();
    let mut stop = pin!(stop);
    loop {
        tokio::select! {
            _ = &mut stop => break,
            accepted = listener.accept() => {
                let (stream, remote) = match accepted {
                    Ok(accepted) => accepted,
                    Err(error) => {
                        tracing::warn!(%error, "a connection could not be accepted");
                        continue;
                    }
                };
                let service = router.clone().layer(Extension(ConnectInfo(remote)));
                connections.spawn(connection(stream, service, closed.clone(), limits.header_read));
            }
            Some(_) = connections.join_next(), if !connections.is_empty() => {}
        }
    }
    drop(listener);
    let _ = closing.send(true);
    let drained = tokio::time::timeout(limits.drain, async {
        while connections.join_next().await.is_some() {}
    })
    .await;
    if drained.is_err() {
        tracing::warn!(
            open = connections.len(),
            "connections still open after {} seconds; closing them",
            limits.drain.as_secs()
        );
        connections.abort_all();
    }
    Ok(())
}

async fn connection(
    stream: tokio::net::TcpStream,
    service: Router,
    mut closed: watch::Receiver<bool>,
    header_read: Duration,
) {
    let mut builder = http1::Builder::new();
    builder
        .timer(TokioTimer::new())
        .header_read_timeout(header_read);
    let served = builder
        .serve_connection(TokioIo::new(stream), TowerToHyperService::new(service))
        .with_upgrades();
    let mut served = pin!(served);
    tokio::select! {
        result = served.as_mut() => {
            if let Err(error) = result {
                tracing::debug!(%error, "a connection ended with an error");
            }
            return;
        }
        _ = closed.wait_for(|closed| *closed) => {}
    }
    served.as_mut().graceful_shutdown();
    if let Err(error) = served.await {
        tracing::debug!(%error, "a connection ended with an error");
    }
}
