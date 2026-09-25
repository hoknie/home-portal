use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;

use super::shutdown;
use crate::types::{BootError, Registry};

pub async fn serve(
    listener: TcpListener,
    router: Router,
    registry: &Registry,
) -> Result<(), BootError> {
    let address = listener.local_addr().map_err(|source| BootError::Serve {
        address: SocketAddr::from(([0, 0, 0, 0], 0)),
        source,
    })?;
    tracing::info!(%address, "listening");
    let served = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown::requested())
    .await
    .map_err(|source| BootError::Serve { address, source });
    for feature in &registry.features {
        feature.stop();
    }
    tracing::info!("stopped");
    served
}
