use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use portal_auth::hash_password;

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn health(port: u16) -> Option<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .write_all(b"GET /health HTTP/1.1\r\nhost: localhost\r\nconnection: close\r\n\r\n")
        .ok()?;
    let mut answer = String::new();
    stream.read_to_string(&mut answer).ok()?;
    Some(answer)
}

#[test]
fn the_serve_command_starts_the_portal_and_stops_on_a_signal() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let hash = hash_password("secret").unwrap();
    fs::write(
        &path,
        format!("[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n"),
    )
    .unwrap();
    let port = free_port();
    let mut child = Command::new(BINARY)
        .arg("serve")
        .env("HOME_PORTAL_CONFIG", &path)
        .env("HOME_PORTAL_ADDRESS", format!("127.0.0.1:{port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let began = Instant::now();
    let answer = loop {
        if let Some(answer) = health(port) {
            break answer;
        }
        assert!(child.try_wait().unwrap().is_none(), "the portal exited");
        assert!(
            began.elapsed() < Duration::from_secs(20),
            "the portal did not start"
        );
        thread::sleep(Duration::from_millis(50));
    };
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.ends_with("ok"), "{answer}");
    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    assert!(child.wait().unwrap().success());
}
