use std::fs;
use std::net::{IpAddr, SocketAddr, TcpListener as StdTcp, UdpSocket as StdUdp};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use hickory_proto::op::{Message, Query, ResponseCode};
use hickory_proto::rr::{Name, RecordType};
use ipnet::IpNet;
use portal_config::ConfigStore;
use portal_model::{Environment, Environments};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, ServerName};
use tempfile::TempDir;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

use super::DnsRuntime;
use crate::ports::DnsSources;
use crate::services::Library;
use crate::types::{Cadence, PublishedHost};

struct Sources;

impl DnsSources for Sources {
    fn environments(&self) -> Environments {
        Environments::new(vec![(
            Environment::parse("local").unwrap(),
            vec!["127.0.0.0/8".parse::<IpNet>().unwrap()],
        )])
    }

    fn published(&self) -> Vec<PublishedHost> {
        vec![PublishedHost {
            host: "jellyfin.home".into(),
            environments: None,
        }]
    }

    fn interfaces(&self) -> Vec<IpAddr> {
        Vec::new()
    }
}

const QUICK: Cadence = Cadence {
    tick: Duration::from_millis(50),
    retry: Duration::from_millis(200),
    interfaces: Duration::from_secs(60),
    idle: Duration::from_millis(300),
};

fn free_port() -> u16 {
    let tcp = StdTcp::bind("127.0.0.1:0").unwrap();
    let port = tcp.local_addr().unwrap().port();
    drop(tcp);
    port
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/loops/fixtures")
        .join(name)
}

fn configuration(port: u16, extra: &str) -> String {
    format!(
        "[environments.local]\nnetworks = [\"127.0.0.0/8\"]\n\n[proxy]\nenabled = true\nportal_host = \"portal.home\"\n\n[dns]\nenabled = true\naddress = \"127.0.0.1\"\nport = {port}\nzones = [\"home\"]\n\n[dns.addresses]\nlocal = \"192.168.1.60\"\n{extra}"
    )
}

fn started(text: &str) -> (TempDir, Arc<DnsRuntime>) {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let runtime = Arc::new(DnsRuntime {
        configuration: Arc::new(ConfigStore::open(&path).unwrap()),
        sources: Arc::new(Sources),
        library: Arc::new(Library::default()),
        cadence: QUICK,
    });
    tokio::spawn(runtime.clone().run());
    (folder, runtime)
}

fn query(name: &str, record_type: RecordType) -> Vec<u8> {
    let mut message = Message::new();
    message
        .set_id(9)
        .add_query(Query::query(Name::from_ascii(name).unwrap(), record_type));
    message.to_vec().unwrap()
}

async fn over_udp(port: u16, bytes: &[u8]) -> Option<Message> {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.send_to(bytes, ("127.0.0.1", port)).await.ok()?;
    let mut buffer = vec![0u8; 4096];
    let (size, _) = tokio::time::timeout(Duration::from_millis(300), socket.recv_from(&mut buffer))
        .await
        .ok()?
        .ok()?;
    Message::from_vec(&buffer[..size]).ok()
}

async fn framed<S: AsyncRead + AsyncWrite + Unpin>(stream: &mut S, bytes: &[u8]) -> Message {
    let mut out = (bytes.len() as u16).to_be_bytes().to_vec();
    out.extend(bytes);
    stream.write_all(&out).await.unwrap();
    let mut length = [0u8; 2];
    stream.read_exact(&mut length).await.unwrap();
    let mut answer = vec![0u8; usize::from(u16::from_be_bytes(length))];
    stream.read_exact(&mut answer).await.unwrap();
    Message::from_vec(&answer).unwrap()
}

async fn eventually(mut check: impl AsyncFnMut() -> bool) -> bool {
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(5) {
        if check().await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test(flavor = "multi_thread")]
async fn udp_and_tcp_answer_from_the_book() {
    let port = free_port();
    let (_folder, _runtime) = started(&configuration(port, ""));
    let bytes = query("jellyfin.home.", RecordType::A);
    assert!(eventually(async || over_udp(port, &bytes).await.is_some()).await);
    let answer = over_udp(port, &bytes).await.unwrap();
    assert_eq!(answer.answers()[0].data().to_string(), "192.168.1.60");
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let over_tcp = framed(&mut stream, &bytes).await;
    assert_eq!(over_tcp.answers()[0].data().to_string(), "192.168.1.60");
    let transfer = framed(&mut stream, &query("home.", RecordType::AXFR)).await;
    assert_eq!(transfer.response_code(), ResponseCode::Refused);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_slow_connection_is_closed_after_the_idle_time() {
    let port = free_port();
    let (_folder, runtime) = started(&configuration(port, ""));
    assert!(eventually(async || runtime.library.state().plain.listening).await);
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let began = Instant::now();
    let mut buffer = [0u8; 1];
    let read = tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer))
        .await
        .unwrap();
    assert!(matches!(read, Ok(0) | Err(_)));
    assert!(began.elapsed() >= QUICK.idle);
}

#[tokio::test(flavor = "multi_thread")]
async fn changing_the_port_moves_the_server() {
    let (first, second) = (free_port(), free_port());
    let (folder, runtime) = started(&configuration(first, ""));
    assert!(eventually(async || runtime.library.state().plain.listening).await);
    fs::write(
        folder.path().join("home-portal.toml"),
        configuration(second, ""),
    )
    .unwrap();
    let bytes = query("jellyfin.home.", RecordType::A);
    assert!(eventually(async || over_udp(second, &bytes).await.is_some()).await);
    assert!(over_udp(first, &bytes).await.is_none());
    assert_eq!(
        runtime.library.state().plain.address,
        Some(SocketAddr::from(([127, 0, 0, 1], second)))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_port_in_use_is_reported_without_stopping_anything() {
    let holder = StdUdp::bind("127.0.0.1:0").unwrap();
    let port = holder.local_addr().unwrap().port();
    let (_folder, runtime) = started(&configuration(port, ""));
    assert!(
        eventually(async || runtime
            .library
            .state()
            .plain
            .reason
            .is_some_and(|reason| reason.contains(&port.to_string())))
        .await
    );
    let state = runtime.library.state();
    assert!(!state.plain.listening);
    assert!(state.last_error.is_some());
}

fn trusting(certificate: &Path) -> tokio_rustls::TlsConnector {
    let mut roots = rustls::RootCertStore::empty();
    for found in CertificateDer::pem_file_iter(certificate).unwrap() {
        roots.add(found.unwrap()).unwrap();
    }
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
}

async fn over_tls(port: u16, trusted: &Path) -> Option<Message> {
    let stream = TcpStream::connect(("127.0.0.1", port)).await.ok()?;
    let mut secured = trusting(trusted)
        .connect(ServerName::try_from("portal.home").unwrap(), stream)
        .await
        .ok()?;
    Some(framed(&mut secured, &query("jellyfin.home.", RecordType::A)).await)
}

#[tokio::test(flavor = "multi_thread")]
async fn dns_over_tls_answers_and_follows_a_replaced_certificate() {
    let (port, tls) = (free_port(), free_port());
    let folder = TempDir::new().unwrap();
    let (certificate, key) = (folder.path().join("dns.crt"), folder.path().join("dns.key"));
    fs::copy(fixture("first.crt"), &certificate).unwrap();
    fs::copy(fixture("first.key"), &key).unwrap();
    let extra = format!(
        "\n[dns.tls]\nenabled = true\nport = {tls}\ncertificate = \"{}\"\nkey = \"{}\"\n",
        certificate.display(),
        key.display()
    );
    let (_config, runtime) = started(&configuration(port, &extra));
    assert!(eventually(async || runtime.library.state().tls.listening).await);
    let answer = over_tls(tls, &fixture("first.crt")).await.unwrap();
    assert_eq!(answer.answers()[0].data().to_string(), "192.168.1.60");
    std::thread::sleep(Duration::from_millis(1100));
    fs::copy(fixture("second.crt"), &certificate).unwrap();
    fs::copy(fixture("second.key"), &key).unwrap();
    assert!(eventually(async || over_tls(tls, &fixture("second.crt")).await.is_some()).await);
}
