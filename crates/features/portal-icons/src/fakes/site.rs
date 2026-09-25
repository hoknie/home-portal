use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl Reply {
    pub fn page(body: &str) -> Reply {
        Reply {
            status: 200,
            content_type: "text/html",
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn image(body: &[u8]) -> Reply {
        Reply {
            status: 200,
            content_type: "image/png",
            body: body.to_vec(),
        }
    }

    pub fn missing() -> Reply {
        Reply {
            status: 404,
            content_type: "text/plain",
            body: b"nope".to_vec(),
        }
    }
}

pub struct Site {
    pub address: SocketAddr,
    hits: Arc<AtomicUsize>,
}

impl Site {
    pub async fn start(replies: BTreeMap<String, Reply>) -> Site {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let replies = replies.clone();
                let counter = counter.clone();
                tokio::spawn(async move {
                    let mut buffer = [0u8; 2048];
                    let read = socket.read(&mut buffer).await.unwrap_or_default();
                    counter.fetch_add(1, Ordering::SeqCst);
                    let request = String::from_utf8_lossy(&buffer[..read]).to_string();
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("/")
                        .to_string();
                    let reply = replies.get(&path).cloned().unwrap_or_else(Reply::missing);
                    let mut bytes = format!(
                        "HTTP/1.1 {} X\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                        reply.status,
                        reply.content_type,
                        reply.body.len()
                    )
                    .into_bytes();
                    bytes.extend_from_slice(&reply.body);
                    let _ = socket.write_all(&bytes).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        Site { address, hits }
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }
}
