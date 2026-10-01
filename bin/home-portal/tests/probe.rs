use std::net::TcpListener;
use std::process::Command;

use portal_testing::{Answer, FakeHttp};

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");

async fn answering_http() -> u16 {
    FakeHttp::always(Answer::status(200).with_body("ok"))
        .await
        .address
        .port()
}

fn closed_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[tokio::test(flavor = "multi_thread")]
async fn probing_a_url_that_answers_prints_up_and_exits_zero() {
    let port = answering_http().await;
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

#[tokio::test(flavor = "multi_thread")]
async fn probing_a_configured_service_uses_its_address_and_settings() {
    let port = answering_http().await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    std::fs::write(
        &path,
        format!(
            "[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://127.0.0.1:{port}\"\nprobe = {{ path = \"/health\" }}\n"
        ),
    )
    .unwrap();
    portal_testing::split(&path);
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
fn an_unknown_kind_is_a_usage_error_that_lists_the_kinds() {
    let output = Command::new(BINARY)
        .args(["probe", "http://127.0.0.1:1", "--kind", "udp"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8_lossy(&output.stderr);
    for word in ["udp", "http", "tcp", "icmp"] {
        assert!(error.contains(word), "{error}");
    }
}

#[test]
fn an_unknown_service_id_fails_with_an_error_line_and_status_one() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    std::fs::write(&path, "").unwrap();
    portal_testing::split(&path);
    let output = Command::new(BINARY)
        .args(["probe", "nothing"])
        .env("HOME_PORTAL_CONFIG", &path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.starts_with("error: "), "{error}");
    assert!(error.contains("\"nothing\""), "{error}");
    assert!(error.contains(&path.display().to_string()), "{error}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_piped_probe_report_has_no_escape_sequences() {
    let port = answering_http().await;
    let output = Command::new(BINARY)
        .args(["probe", &format!("http://127.0.0.1:{port}")])
        .env_remove("CLICOLOR_FORCE")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.contains(&0x1b));
}

#[test]
fn a_forced_colour_probe_report_colours_the_state() {
    let port = closed_port();
    let output = Command::new(BINARY)
        .args(["probe", &format!("tcp://127.0.0.1:{port}"), "--kind", "tcp"])
        .env("CLICOLOR_FORCE", "1")
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("\u{1b}[1m\u{1b}[31mdown"), "{text:?}");
}
