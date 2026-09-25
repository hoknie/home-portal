use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct ForecastService {
    pub address: SocketAddr,
    requests: Arc<Mutex<Vec<String>>>,
}

impl ForecastService {
    pub async fn start(body: &'static str, status: u16) -> ForecastService {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = requests.clone();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let seen = seen.clone();
                tokio::spawn(async move {
                    let mut buffer = [0u8; 4096];
                    let read = socket.read(&mut buffer).await.unwrap_or_default();
                    let request = String::from_utf8_lossy(&buffer[..read]).to_string();
                    seen.lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(request);
                    let reply = format!(
                        "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = socket.write_all(reply.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        ForecastService { address, requests }
    }

    pub fn endpoint(&self) -> String {
        format!("http://{}/v1/forecast", self.address)
    }

    pub fn requests(&self) -> Vec<String> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
