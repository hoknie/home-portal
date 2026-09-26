use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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

fn curl(jar: &Path, arguments: &[&str]) -> (String, String) {
    let output = Command::new("curl")
        .args(["-s", "-o", "-", "-w", "\n%{http_code}", "-b"])
        .arg(jar)
        .arg("-c")
        .arg(jar)
        .args(arguments)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let (body, status) = text.rsplit_once('\n').unwrap_or(("", "000"));
    (status.to_string(), body.to_string())
}

fn configuration(port: u16, hash: &str) -> String {
    format!(
        "[network]\naddress = \"127.0.0.1\"\nport = {port}\n\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n\n[[automations]]\nid = \"on-stop\"\ntitle = \"On stop\"\nwhen = {{ event = \"portal.stopping\" }}\nrun = {{ script = \"stop.sh\", timeout_seconds = 30 }}\n"
    )
}

fn wait_for(what: &str, mut check: impl FnMut() -> bool) {
    let began = Instant::now();
    while !check() {
        assert!(began.elapsed() < Duration::from_secs(15), "{what}");
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn a_restart_from_the_api_applies_the_new_port_in_the_same_process_and_runs_the_stop_automation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let (first, second) = (free_port(), free_port());
    let hash = hash_password("secret").unwrap();
    fs::write(&path, configuration(first, &hash)).unwrap();
    let scripts = directory.path().join("scripts");
    fs::create_dir(&scripts).unwrap();
    fs::set_permissions(&scripts, fs::Permissions::from_mode(0o755)).unwrap();
    let script = scripts.join("stop.sh");
    fs::write(&script, "#!/bin/sh\necho stopped >> stops\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut child = Command::new(BINARY)
        .env("HOME_PORTAL_CONFIG", &path)
        .env_remove("HOME_PORTAL_ADDRESS")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let jar = directory.path().join("cookies");
    let at = |port: u16, route: &str| format!("http://127.0.0.1:{port}{route}");
    wait_for("the portal never answered", || {
        curl(&jar, &[&at(first, "/health")]).0 == "200"
    });
    let (status, _) = curl(
        &jar,
        &[
            "-H",
            "content-type: application/json",
            "-d",
            r#"{"name":"admin","password":"secret"}"#,
            &at(first, "/api/session"),
        ],
    );
    assert_eq!(status, "200");
    fs::write(&path, configuration(second, &hash)).unwrap();
    let (_, network) = curl(&jar, &[&at(first, "/api/network")]);
    assert!(network.contains("\"restart_required\":true"), "{network}");
    let (status, _) = curl(&jar, &["-X", "POST", &at(first, "/api/portal/restart")]);
    assert_eq!(status, "202");
    wait_for("the portal never came back on the new port", || {
        curl(&jar, &[&at(second, "/health")]).0 == "200"
    });
    assert!(child.try_wait().unwrap().is_none(), "the process exited");
    assert_eq!(curl(&jar, &[&at(first, "/health")]).0, "000");
    let (status, network) = curl(&jar, &[&at(second, "/api/network")]);
    assert_eq!(status, "200", "the session did not survive: {network}");
    assert!(network.contains("\"restart_required\":false"), "{network}");
    let stops = fs::read_to_string(scripts.join("stops")).unwrap();
    assert_eq!(stops.lines().count(), 1, "{stops}");
    let killed = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    assert!(child.wait().unwrap().success());
}
