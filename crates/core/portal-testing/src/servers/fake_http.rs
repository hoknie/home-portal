use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::types::{Answer, Request};

type Replies = Arc<dyn Fn(&Request) -> Answer + Send + Sync>;

pub struct FakeHttp {
    pub address: SocketAddr,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl FakeHttp {
    pub const LONGEST_REQUEST: usize = 1024 * 1024;
    pub const HANG: Duration = Duration::from_secs(3600);

    pub async fn start(reply: impl Fn(&Request) -> Answer + Send + Sync + 'static) -> FakeHttp {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = requests.clone();
        let reply: Replies = Arc::new(reply);
        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                tokio::spawn(answer(socket, reply.clone(), seen.clone()));
            }
        });
        FakeHttp { address, requests }
    }

    pub async fn always(answer: Answer) -> FakeHttp {
        FakeHttp::start(move |_| answer.clone()).await
    }

    pub async fn pages(pages: BTreeMap<String, Answer>) -> FakeHttp {
        FakeHttp::start(move |request| {
            pages
                .get(&request.path)
                .cloned()
                .unwrap_or_else(|| Answer::status(404))
        })
        .await
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn hits(&self) -> usize {
        self.requests().len()
    }
}

async fn answer(mut socket: TcpStream, reply: Replies, seen: Arc<Mutex<Vec<Request>>>) {
    let request = Request::parse(&read_request(&mut socket).await);
    let answer = reply(&request);
    seen.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(request);
    match &answer {
        Answer::Hang => tokio::time::sleep(FakeHttp::HANG).await,
        Answer::Http { delay, .. } => tokio::time::sleep(*delay).await,
        Answer::Raw(_) => {}
    }
    let _ = socket.write_all(&answer.bytes()).await;
    let _ = socket.shutdown().await;
}

async fn read_request(socket: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let Ok(read) = socket.read(&mut buffer).await else {
            return bytes;
        };
        if read == 0 {
            return bytes;
        }
        bytes.extend_from_slice(&buffer[..read]);
        if complete(&bytes) || bytes.len() > FakeHttp::LONGEST_REQUEST {
            return bytes;
        }
    }
}

fn complete(bytes: &[u8]) -> bool {
    if !bytes.first().is_some_and(u8::is_ascii_uppercase) {
        return true;
    }
    let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let length = Request::parse(&bytes[..end + 4])
        .header("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    bytes.len() >= end + 4 + length
}
