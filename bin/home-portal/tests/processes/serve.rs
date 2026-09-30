use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use portal_auth::hash_password;

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");
const QUIET: &str = "[permissions]\nrequest_at_start = false\n\n";

fn listening_port(stdout: ChildStdout) -> mpsc::Receiver<u16> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if !line.contains("listening") {
                continue;
            }
            let port = line
                .rsplit_once("address=")
                .and_then(|(_, address)| address.trim().rsplit_once(':'))
                .and_then(|(_, port)| port.parse().ok());
            if let Some(port) = port {
                let _ = sender.send(port);
            }
        }
    });
    receiver
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
        format!(
            "{QUIET}[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\ngroup = \"admin\"\n"
        ),
    )
    .unwrap();
    let mut child = Command::new(BINARY)
        .arg("serve")
        .env("HOME_PORTAL_CONFIG", &path)
        .env("HOME_PORTAL_ADDRESS", "127.0.0.1:0")
        .env("NO_COLOR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let port = listening_port(child.stdout.take().unwrap())
        .recv_timeout(Duration::from_secs(20))
        .expect("the portal did not report its address");
    let answer = health(port).expect("the portal did not answer");
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.ends_with("ok"), "{answer}");
    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    assert!(child.wait().unwrap().success());
}
