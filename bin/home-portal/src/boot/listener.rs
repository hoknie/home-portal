use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::types::BootError;

pub async fn bind(address: SocketAddr) -> Result<TcpListener, BootError> {
    TcpListener::bind(address)
        .await
        .map_err(|source| BootError::Bind { address, source })
}
