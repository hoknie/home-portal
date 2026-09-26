use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
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

fn fetch(port: u16, route: &str) -> (String, String) {
    let output = Command::new("curl")
        .args(["-s", "-o", "-", "-w", "\n%{http_code}"])
        .arg(format!("http://127.0.0.1:{port}{route}"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let (body, status) = text.rsplit_once('\n').unwrap_or(("", "000"));
    (status.to_string(), body.to_string())
}

fn spawned(program: &Path, configuration: &Path, port: u16) -> Child {
    let began = Instant::now();
    loop {
        let attempt = Command::new(program)
            .env("HOME_PORTAL_CONFIG", configuration)
            .env("HOME_PORTAL_ADDRESS", format!("127.0.0.1:{port}"))
            .env_remove("HOME_PORTAL_WEB")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        match attempt {
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && began.elapsed() < Duration::from_secs(5) =>
            {
                thread::sleep(Duration::from_millis(50));
            }
            result => return result.unwrap(),
        }
    }
}

fn started(program: &Path, configuration: &Path, port: u16) -> Child {
    let child = spawned(program, configuration, port);
    let began = Instant::now();
    while fetch(port, "/health").0 != "200" {
        assert!(
            began.elapsed() < Duration::from_secs(15),
            "the portal never answered"
        );
        thread::sleep(Duration::from_millis(100));
    }
    child
}

fn configuration(directory: &Path) -> std::path::PathBuf {
    let path = directory.join("home-portal.toml");
    let hash = hash_password("secret").unwrap();
    fs::write(
        &path,
        format!("[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n"),
    )
    .unwrap();
    path
}

#[test]
fn the_interface_beside_an_unpacked_binary_is_served_through_a_link() {
    let directory = tempfile::tempdir().unwrap();
    let app = directory.path().join("app");
    fs::create_dir_all(app.join("web/en")).unwrap();
    fs::write(app.join("web/en/index.html"), "<p>from the archive</p>").unwrap();
    let binary = app.join("home-portal");
    fs::copy(BINARY, &binary).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    let links = directory.path().join("bin");
    fs::create_dir(&links).unwrap();
    std::os::unix::fs::symlink(&binary, links.join("home-portal")).unwrap();
    let port = free_port();
    let mut child = started(
        &links.join("home-portal"),
        &configuration(directory.path()),
        port,
    );
    let (status, body) = fetch(port, "/");
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(status, "200");
    assert!(body.contains("from the archive"), "{body}");
}

#[test]
fn without_an_interface_the_api_answers_and_pages_name_the_variable() {
    let directory = tempfile::tempdir().unwrap();
    let port = free_port();
    let mut child = started(Path::new(BINARY), &configuration(directory.path()), port);
    let (status, body) = fetch(port, "/");
    let (health, _) = fetch(port, "/health");
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(health, "200");
    assert_eq!(status, "503");
    assert!(body.contains("HOME_PORTAL_WEB"), "{body}");
}
