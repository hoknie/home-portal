use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use portal_auth::hash_password;

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

#[cfg(unix)]
#[test]
fn stopping_on_a_signal_writes_the_history_where_the_storage_section_points() {
    let port = answering_http();
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("config")).unwrap();
    let path = directory.path().join("config/home-portal.toml");
    let hash = hash_password("secret").unwrap();
    fs::write(
        &path,
        format!(
            "[storage]\ndirectory = \"../env\"\n\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://127.0.0.1:{port}\"\n"
        ),
    )
    .unwrap();
    let mut child = Command::new(BINARY)
        .env("HOME_PORTAL_CONFIG", &path)
        .env("HOME_PORTAL_ADDRESS", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_secs(4));
    let history = directory.path().join("env/history/media.ndjson");
    assert!(!history.exists());
    assert!(directory.path().join("env/icons").is_dir());
    assert!(!directory.path().join("config/icons").exists());
    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    let started = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the portal did not stop"
        );
        thread::sleep(Duration::from_millis(50));
    }
    let text = fs::read_to_string(&history).unwrap();
    assert!(text.contains("\"state\":\"up\""), "{text}");
    assert!(!directory.path().join("config/history").exists());
}

#[cfg(unix)]
fn portal_with_a_stop_script(body: &str) -> (tempfile::TempDir, std::process::Child) {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let hash = hash_password("secret").unwrap();
    fs::write(
        &path,
        format!(
            "[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n\n[[automations]]\nid = \"on-stop\"\ntitle = \"On stop\"\nwhen = {{ event = \"portal.stopping\" }}\nrun = {{ script = \"stop.sh\", timeout_seconds = 120 }}\n"
        ),
    )
    .unwrap();
    let scripts = directory.path().join("scripts");
    fs::create_dir(&scripts).unwrap();
    fs::set_permissions(&scripts, fs::Permissions::from_mode(0o755)).unwrap();
    let script = scripts.join("stop.sh");
    fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let child = Command::new(BINARY)
        .env("HOME_PORTAL_CONFIG", &path)
        .env("HOME_PORTAL_ADDRESS", "127.0.0.1:0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_secs(2));
    (directory, child)
}

#[cfg(unix)]
fn stop(child: &mut std::process::Child) -> (Duration, std::process::ExitStatus) {
    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return (started.elapsed(), status);
        }
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "the portal did not stop"
        );
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
#[test]
fn a_stop_script_completes_before_the_portal_exits() {
    let (directory, mut child) = portal_with_a_stop_script("sleep 2; echo done > stopped");
    let (waited, status) = stop(&mut child);
    assert!(status.success());
    assert!(waited >= Duration::from_secs(2), "{waited:?}");
    assert!(directory.path().join("scripts/stopped").exists());
}

#[cfg(unix)]
#[test]
fn a_stop_script_that_hangs_is_killed_and_the_portal_still_exits_cleanly() {
    let (directory, mut child) = portal_with_a_stop_script("echo $$ > pid; sleep 60");
    let (waited, status) = stop(&mut child);
    assert!(status.success());
    assert!(waited < Duration::from_secs(13), "{waited:?}");
    let pid = fs::read_to_string(directory.path().join("scripts/pid")).unwrap();
    thread::sleep(Duration::from_millis(300));
    let alive = Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(!alive.success());
}

#[cfg(unix)]
#[test]
fn a_stop_script_killed_at_shutdown_is_written_to_the_journal() {
    let (directory, mut child) = portal_with_a_stop_script("sleep 60");
    let (_, status) = stop(&mut child);
    assert!(status.success());
    let journal = fs::read_to_string(directory.path().join("automations/runs.ndjson")).unwrap();
    assert!(journal.contains("\"portal.stopping\""), "{journal}");
}
