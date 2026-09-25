use std::fs;
use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");

const ENABLED: &str = r#"[network]
trusted_proxies = ["127.0.0.1"]

[proxy]
enabled = true
portal_host = "portal.example.com"
cookie_domain = "example.com"

[[services]]
id = "nas"
name = "NAS"
url = "http://192.168.1.5"
proxy = { host = "nas.example.com", auth = ["internet"] }
probe = { enabled = false }
"#;

fn render_with(text: &str) -> Output {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let hash = portal_auth::hash_password("secret").unwrap();
    fs::write(
        &path,
        format!("{text}\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n"),
    )
    .unwrap();
    Command::new(BINARY)
        .args(["proxy", "render"])
        .env("HOME_PORTAL_CONFIG", &path)
        .env_remove("HOME_PORTAL_ADDRESS")
        .output()
        .unwrap()
}

#[test]
fn rendering_prints_the_same_caddy_configuration_every_time() {
    let first = render_with(ENABLED);
    let second = render_with(ENABLED);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    let rendered: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    let hosts: Vec<&str> = rendered["apps"]["http"]["servers"]["home-portal"]["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|route| route["match"][0]["host"][0].as_str().unwrap())
        .collect();
    assert_eq!(hosts, vec!["portal.example.com", "nas.example.com"]);
}

#[test]
fn an_invalid_configuration_is_refused_with_its_error() {
    let output =
        render_with(&ENABLED.replace("trusted_proxies = [\"127.0.0.1\"]", "trusted_proxies = []"));
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("network.trusted_proxies"), "{error}");
}

#[test]
fn a_disabled_proxy_has_nothing_to_render() {
    let output = render_with(&ENABLED.replace("enabled = true", "enabled = false"));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not enabled"));
}
