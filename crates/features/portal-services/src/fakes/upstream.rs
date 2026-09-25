use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Copy)]
pub enum Behaviour {
    Status { code: u16, delay: Duration },
    Hang,
    Garbage,
    Redirect,
}

pub struct Upstream {
    pub address: SocketAddr,
    hits: Arc<AtomicUsize>,
}

impl Upstream {
    pub async fn start(behaviour: Behaviour) -> Upstream {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buffer = [0u8; 1024];
                    let _ = socket.read(&mut buffer).await;
                    let reply: Vec<u8> = match behaviour {
                        Behaviour::Status { code, delay } => {
                            tokio::time::sleep(delay).await;
                            format!("HTTP/1.1 {code} X\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok").into_bytes()
                        }
                        Behaviour::Hang => {
                            tokio::time::sleep(Duration::from_secs(3600)).await;
                            Vec::new()
                        }
                        Behaviour::Garbage => b"this is not http at all\r\n\r\n".to_vec(),
                        Behaviour::Redirect => {
                            b"HTTP/1.1 302 Found\r\nlocation: http://127.0.0.1:1/\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_vec()
                        }
                    };
                    let _ = socket.write_all(&reply).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        Upstream { address, hits }
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }
}
