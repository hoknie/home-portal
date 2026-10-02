use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use portal_auth::hash_password;

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");
const QUIET: &str = "[permissions]\nrequest_at_start = false\n";

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn curl(port: u16, route: &str) -> (String, String) {
    let output = Command::new("curl")
        .args(["-s", "-o", "-", "-w", "\n%{http_code}"])
        .arg(format!("http://127.0.0.1:{port}{route}"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let (body, status) = text.rsplit_once('\n').unwrap_or(("", "000"));
    (status.to_string(), body.to_string())
}

fn wait_for(what: &str, mut check: impl FnMut() -> bool) {
    let began = Instant::now();
    while !check() {
        assert!(began.elapsed() < Duration::from_secs(20), "{what}");
        thread::sleep(Duration::from_millis(100));
    }
}

fn admin() -> String {
    let hash = hash_password("secret").unwrap();
    format!("[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\ngroup = \"admin\"\n")
}

struct Portal {
    directory: tempfile::TempDir,
    port: u16,
    child: Child,
}

impl Portal {
    fn start(main: &str, files: &[(&str, &str)]) -> Portal {
        let directory = tempfile::tempdir().unwrap();
        let port = free_port();
        fs::write(
            directory.path().join("home-portal.toml"),
            format!("{QUIET}\n[network]\naddress = \"127.0.0.1\"\nport = {port}\n\n{main}"),
        )
        .unwrap();
        for (name, text) in files {
            fs::write(directory.path().join(name), text).unwrap();
        }
        let errors = fs::File::create(directory.path().join("stderr")).unwrap();
        let child = Command::new(BINARY)
            .arg("serve")
            .env(
                "HOME_PORTAL_CONFIG",
                directory.path().join("home-portal.toml"),
            )
            .env_remove("HOME_PORTAL_ADDRESS")
            .env("NO_COLOR", "1")
            .stdout(Stdio::null())
            .stderr(errors)
            .spawn()
            .unwrap();
        let portal = Portal {
            directory,
            port,
            child,
        };
        wait_for("the portal never answered", || {
            curl(portal.port, "/health").0 != "000"
        });
        portal
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.path().join(name)
    }

    fn report(&self) -> serde_json::Value {
        let (status, body) = curl(self.port, "/api/portal/failure");
        assert_eq!(status, "200", "{body}");
        serde_json::from_str(&body).unwrap()
    }

    fn fields(&self) -> Vec<String> {
        self.report()["problems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|problem| problem["field"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    fn stop(mut self) {
        let killed = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(killed.success());
        assert!(self.child.wait().unwrap().success());
    }
}

fn rewrite(path: &Path, text: &str) {
    thread::sleep(Duration::from_millis(20));
    fs::write(path, text).unwrap();
}

#[test]
fn a_section_in_the_wrong_file_keeps_the_process_up_and_explains_itself() {
    let portal = Portal::start(&admin(), &[]);
    assert_eq!(
        curl(portal.port, "/health"),
        ("503".to_string(), "failed".to_string())
    );
    assert_eq!(portal.fields(), ["users"]);
    assert_eq!(portal.report()["details"], true);
    let (status, _) = curl(portal.port, "/admin/services/");
    assert_eq!(status, "307");
    let (status, body) = curl(portal.port, "/api/services");
    assert_eq!(status, "503");
    assert!(body.contains("/fatal/"), "{body}");
    let errors = fs::read_to_string(portal.path("stderr")).unwrap();
    assert!(
        errors.contains("error: ")
            && errors.contains("users: is in home-portal.toml; it belongs in users.toml"),
        "{errors}"
    );
    portal.stop();
}

#[test]
fn a_fixed_configuration_brings_the_portal_up_in_the_same_process() {
    let users = admin();
    let portal = Portal::start(
        "",
        &[
            ("users.toml", &users),
            ("proxy.toml", "[proxy]\nenabled = false\n"),
        ],
    );
    assert_eq!(portal.fields(), ["proxy.enabled"]);
    let message = portal.report()["problems"][0]["message"].to_string();
    assert!(message.contains("modules.proxy"), "{message}");
    let process = portal.child.id();
    rewrite(&portal.path("proxy.toml"), "[proxy]\n");
    wait_for("the portal did not recover", || {
        curl(portal.port, "/health") == ("200".to_string(), "ok".to_string())
    });
    assert_eq!(portal.child.id(), process);
    let (status, _) = curl(portal.port, "/fatal/");
    assert_eq!(status, "307");
    assert_eq!(curl(portal.port, "/api/portal/failure").0, "404");
    portal.stop();
}

#[test]
fn a_fix_that_is_not_enough_leaves_only_the_remaining_problem() {
    let users = admin();
    let portal = Portal::start(
        "",
        &[
            ("users.toml", &users),
            ("proxy.toml", "[proxy]\nenabled = false\n"),
            ("dns.toml", "[dns]\nenabled = false\n"),
        ],
    );
    let mut fields = portal.fields();
    fields.sort();
    assert_eq!(fields, ["dns.enabled", "proxy.enabled"]);
    rewrite(&portal.path("proxy.toml"), "[proxy]\n");
    wait_for("the report did not follow the fix", || {
        portal.fields() == ["dns.enabled"]
    });
    assert_eq!(curl(portal.port, "/health").0, "503");
    portal.stop();
}

#[test]
fn a_missing_configuration_file_still_ends_the_process() {
    let directory = tempfile::tempdir().unwrap();
    let output = Command::new(BINARY)
        .arg("serve")
        .env("HOME_PORTAL_CONFIG", directory.path().join("missing.toml"))
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not exist"));
}
