use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");

fn answering_http() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nok");
        }
    });
    port
}

fn closed_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn probing_a_url_that_answers_prints_up_and_exits_zero() {
    let port = answering_http();
    let output = Command::new(BINARY)
        .args(["probe", &format!("http://127.0.0.1:{port}")])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{text}");
    assert!(text.contains("state      up"), "{text}");
    assert!(text.contains("kind       http"), "{text}");
}

#[test]
fn probing_a_closed_port_prints_the_diagnosis_and_advice_and_exits_non_zero() {
    let port = closed_port();
    let output = Command::new(BINARY)
        .args(["probe", &format!("tcp://127.0.0.1:{port}"), "--kind", "tcp"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "{text}");
    assert!(text.contains("state      down"), "{text}");
    assert!(text.contains("diagnosis  refused"), "{text}");
    assert!(text.contains("advice     "), "{text}");
}

#[test]
fn probing_a_configured_service_uses_its_address_and_settings() {
    let port = answering_http();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    std::fs::write(
        &path,
        format!(
            "[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://127.0.0.1:{port}\"\nprobe = {{ path = \"/health\" }}\n"
        ),
    )
    .unwrap();
    let output = Command::new(BINARY)
        .args(["probe", "media"])
        .env("HOME_PORTAL_CONFIG", &path)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{text}");
    assert!(
        text.contains(&format!("target     http://127.0.0.1:{port}/health")),
        "{text}"
    );
}

#[test]
fn an_unknown_service_or_kind_is_refused_with_usage() {
    let output = Command::new(BINARY)
        .args(["probe", "http://127.0.0.1:1", "--kind", "udp"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
}
