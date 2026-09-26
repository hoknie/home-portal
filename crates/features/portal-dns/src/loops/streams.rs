use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;

use crate::services::{Library, handle};
use crate::types::Cadence;

pub const LARGEST_STREAM_ANSWER: usize = u16::MAX as usize;

pub async fn serve_streams(
    listener: TcpListener,
    library: Arc<Library>,
    acceptor: Option<TlsAcceptor>,
    idle: Duration,
) {
    let permits = Arc::new(Semaphore::new(Cadence::CONNECTIONS));
    loop {
        let Ok((stream, peer)) = listener.accept().await else {
            continue;
        };
        let Ok(permit) = permits.clone().try_acquire_owned() else {
            continue;
        };
        let library = library.clone();
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let _permit = permit;
            match acceptor {
                None => converse(stream, peer.ip(), &library, idle).await,
                Some(acceptor) => {
                    if let Ok(Ok(secured)) = timeout(idle, acceptor.accept(stream)).await {
                        converse(secured, peer.ip(), &library, idle).await;
                    }
                }
            }
        });
    }
}

async fn converse<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    peer: IpAddr,
    library: &Library,
    idle: Duration,
) {
    loop {
        let mut length = [0u8; 2];
        if !matches!(
            timeout(idle, stream.read_exact(&mut length)).await,
            Ok(Ok(_))
        ) {
            return;
        }
        let mut message = vec![0u8; usize::from(u16::from_be_bytes(length))];
        if !matches!(
            timeout(idle, stream.read_exact(&mut message)).await,
            Ok(Ok(_))
        ) {
            return;
        }
        let Some(reply) = handle(&library.book(), peer, &message, Some(LARGEST_STREAM_ANSWER))
        else {
            return;
        };
        let Ok(size) = u16::try_from(reply.len()) else {
            return;
        };
        let mut framed = size.to_be_bytes().to_vec();
        framed.extend(reply);
        if stream.write_all(&framed).await.is_err() {
            return;
        }
    }
}
