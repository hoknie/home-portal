use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;

use super::{lifecycle, shutdown};
use crate::types::{BootError, Ended, Registry, Signals};

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
    let served = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown({
        let restart = registry.restart.clone();
        async move {
            shutdown::requested(signals, restart).await;
        }
    })
    .await
    .map_err(|source| BootError::Serve { address, source });
    lifecycle::stopping(registry, address).await;
    for feature in &registry.features {
        feature.stop();
    }
    tracing::info!("stopped");
    served.map(|()| {
        if registry.restart.requested() {
            Ended::Restart
        } else {
            Ended::Stopped
        }
    })
}
