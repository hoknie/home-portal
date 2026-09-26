use std::fs;
use std::net::{TcpListener, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use portal_auth::hash_password;

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");
const TYPE_A: u16 = 1;
const TYPE_SOA: u16 = 6;
const SERIAL_FROM_END: usize = 20;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn question(name: &str, kind: u16) -> Vec<u8> {
    let mut bytes = vec![0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
    for label in name.split('.') {
        bytes.push(label.len() as u8);
        bytes.extend(label.as_bytes());
    }
    bytes.extend([0, (kind >> 8) as u8, kind as u8, 0, 1]);
    bytes
}

fn ask(port: u16, name: &str, kind: u16) -> Option<Vec<u8>> {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    socket
        .send_to(&question(name, kind), ("127.0.0.1", port))
        .ok()?;
    let mut buffer = [0u8; 1024];
    let size = socket.recv(&mut buffer).ok()?;
    Some(buffer[..size].to_vec())
}

fn address_of(answer: &[u8]) -> Option<String> {
    let answers = u16::from_be_bytes([answer[6], answer[7]]);
    (answers > 0 && answer[3] & 0x0f == 0).then(|| {
        let end = answer.len();
        format!(
            "{}.{}.{}.{}",
            answer[end - 4],
            answer[end - 3],
            answer[end - 2],
            answer[end - 1]
        )
    })
}

fn serial_of(answer: &[u8]) -> u32 {
    let at = answer.len() - SERIAL_FROM_END;
    u32::from_be_bytes([answer[at], answer[at + 1], answer[at + 2], answer[at + 3]])
}

fn configuration(dns: u16, admin: u16, hash: &str, photos: &str) -> String {
    format!(
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[environments.local]\nnetworks = [\"127.0.0.0/8\"]\n\n[proxy]\nenabled = true\nportal_host = \"portal.home\"\nadmin = \"http://127.0.0.1:{admin}\"\n\n[dns]\nenabled = true\naddress = \"127.0.0.1\"\nport = {dns}\nzones = [\"home\"]\n\n[dns.addresses]\nlocal = \"192.168.1.60\"\n\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n\n[[services]]\nid = \"photos\"\nname = \"Photos\"\nurl = \"http://192.168.1.7\"\n{photos}"
    )
}

fn eventually(mut check: impl FnMut() -> bool) -> bool {
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(10) {
        if check() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn publishing_a_service_updates_dns_within_seconds() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let (dns, admin) = (free_port(), free_port());
    let hash = hash_password("secret").unwrap();
    fs::write(&path, configuration(dns, admin, &hash, "")).unwrap();
    let mut child = Command::new(BINARY)
        .env("HOME_PORTAL_CONFIG", &path)
        .env("HOME_PORTAL_ADDRESS", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let portal_answered = eventually(|| {
        ask(dns, "portal.home", TYPE_A).and_then(|answer| address_of(&answer))
            == Some("192.168.1.60".into())
    });
    let before = ask(dns, "home", TYPE_SOA).map(|answer| serial_of(&answer));
    let absent = ask(dns, "photos.home", TYPE_A).and_then(|answer| address_of(&answer));
    fs::write(
        &path,
        configuration(dns, admin, &hash, "proxy = { host = \"photos.home\" }\n"),
    )
    .unwrap();
    let began = Instant::now();
    let published = eventually(|| {
        ask(dns, "photos.home", TYPE_A).and_then(|answer| address_of(&answer))
            == Some("192.168.1.60".into())
    });
    let waited = began.elapsed();
    let after = ask(dns, "home", TYPE_SOA).map(|answer| serial_of(&answer));
    let _ = child.kill();
    let _ = child.wait();
    assert!(portal_answered, "the portal host never resolved");
    assert_eq!(absent, None);
    assert!(published, "the new host never resolved");
    assert!(waited < Duration::from_secs(2), "{waited:?}");
    assert!(
        before.is_some() && after.is_some() && before != after,
        "{before:?} {after:?}"
    );
}
