use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::services::{Library, UDP_LIMIT, handle};
use crate::types::Cadence;

pub async fn serve_datagrams(socket: UdpSocket, library: Arc<Library>) {
    let mut buffer = vec![0u8; Cadence::LARGEST_MESSAGE];
    loop {
        let Ok((size, peer)) = socket.recv_from(&mut buffer).await else {
            continue;
        };
        if let Some(reply) = handle(&library.book(), peer.ip(), &buffer[..size], Some(UDP_LIMIT)) {
            let _ = socket.send_to(&reply, peer).await;
        }
    }
}
