use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portal_config::ConfigStore;
use portal_feature::{FieldError, StatusChange, StatusObserver};
use portal_model::{Environment, ProbeOutcome, ServiceState};
use time::OffsetDateTime;
use toml_edit::DocumentMut;

use super::{StatusBoard, Supervisor, check_entry, validate_services};
use crate::fakes::{Behaviour, Upstream};
use crate::probes::Probe;
use crate::types::{Known, ProbeKind, ProbeSettings, ServiceEntry, Wake};

fn entry() -> ServiceEntry {
    ServiceEntry::new("media", "Media", "http://10.0.0.5:8096")
}

fn fields(errors: Vec<FieldError>) -> Vec<String> {
    errors.into_iter().map(|error| error.field).collect()
}

#[test]
fn a_service_with_only_id_name_and_url_gets_the_default_probe() {
    let document: DocumentMut =
        "[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://10.0.0.5\"\n"
            .parse()
            .unwrap();
    assert!(validate_services(&document).is_empty());
    let section = crate::types::ServicesSection::read(&document).unwrap();
    assert_eq!(section.services[0].probe, ProbeSettings::default());
}

#[test]
fn every_rule_names_its_field() {
    let mut bad = entry();
    bad.id = "Bad Id".into();
    bad.name = String::new();
    bad.url = "ftp://x".into();
    bad.description = Some("x".repeat(201));
    bad.probe.path = "health".into();
    bad.probe.every_seconds = 4;
    bad.probe.degraded_after_milliseconds = 0;
    assert_eq!(
        fields(check_entry(&bad, &Known::default())),
        vec![
            "id",
            "name",
            "url",
            "description",
            "probe.path",
            "probe.every_seconds",
            "probe.timeout_seconds",
            "probe.degraded_after_milliseconds"
        ]
    );
}

#[test]
fn a_timeout_must_be_shorter_than_the_period() {
    let mut slow = entry();
    slow.probe.every_seconds = 10;
    slow.probe.timeout_seconds = 10;
    assert_eq!(
        fields(check_entry(&slow, &Known::default())),
        vec!["probe.timeout_seconds"]
    );
}

#[test]
fn a_url_without_a_host_or_with_another_scheme_is_refused() {
    for url in ["not a url", "ftp://nas.local", "mailto:me@example.com"] {
        let mut bad = entry();
        bad.url = url.into();
        assert_eq!(
            fields(check_entry(&bad, &Known::default())),
            vec!["url"],
            "{url}"
        );
    }
}

#[test]
fn duplicate_ids_in_the_file_are_refused_by_entry() {
    let text = "[[services]]\nid = \"a\"\nname = \"A\"\nurl = \"http://a\"\n\n[[services]]\nid = \"a\"\nname = \"B\"\nurl = \"http://b\"\n";
    let document: DocumentMut = text.parse().unwrap();
    assert_eq!(fields(validate_services(&document)), vec!["services[1].id"]);
}

async fn eventually(check: impl Fn() -> bool) -> bool {
    for _ in 0..50 {
        if check() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
    check()
}

fn write(path: &std::path::Path, text: &str) {
    let before = fs::metadata(path)
        .map(|metadata| metadata.modified().unwrap())
        .ok();
    fs::write(path, text).unwrap();
    if let Some(before) = before {
        let file = fs::File::options().write(true).open(path).unwrap();
        file.set_modified(before + Duration::from_secs(2)).unwrap();
    }
}

fn service_text(id: &str, url: &str) -> String {
    format!(
        "[[services]]\nid = \"{id}\"\nname = \"{id}\"\nurl = \"{url}\"\nprobe = {{ every_seconds = 5, timeout_seconds = 1 }}\n"
    )
}

#[tokio::test]
async fn the_supervisor_starts_stops_and_restarts_probing_as_the_file_changes() {
    let first = Upstream::start(Behaviour::Status {
        code: 200,
        delay: Duration::ZERO,
    })
    .await;
    let second = Upstream::start(Behaviour::Status {
        code: 200,
        delay: Duration::ZERO,
    })
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    write(&path, &service_text("media", &first.url()));
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let board = Arc::new(StatusBoard::watched(OffsetDateTime::now_utc(), Vec::new()));
    let supervisor = Supervisor::new(
        store,
        Environment::internet(),
        board.clone(),
        Arc::new(Probe::new().unwrap()),
    );

    supervisor.reconcile();
    assert!(eventually(|| board.status("media").state == ServiceState::Up).await);
    assert_eq!(first.hits(), 1);

    write(&path, &service_text("media", &second.url()));
    supervisor.reconcile();
    assert!(eventually(|| second.hits() == 1).await);

    write(&path, "");
    supervisor.reconcile();
    assert!(supervisor.probing().is_empty());
    assert_eq!(board.status("media").state, ServiceState::Unknown);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(second.hits(), 1);
}

#[tokio::test]
async fn the_probe_goes_to_the_address_of_the_environment_the_portal_is_in() {
    let reachable = Upstream::start(Behaviour::Status {
        code: 200,
        delay: Duration::ZERO,
    })
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let text = format!(
        "[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://127.0.0.1:1\"\naddresses = {{ local = \"{}\", internet = \"http://127.0.0.1:2\" }}\nprobe = {{ every_seconds = 5, timeout_seconds = 1 }}\n",
        reachable.url()
    );
    write(&path, &text);
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let board = Arc::new(StatusBoard::watched(OffsetDateTime::now_utc(), Vec::new()));
    let local = Environment::parse("local").unwrap();
    let supervisor = Supervisor::new(store, local, board.clone(), Arc::new(Probe::new().unwrap()));
    supervisor.reconcile();
    assert!(eventually(|| board.status("nas").state == ServiceState::Up).await);
    assert_eq!(reachable.hits(), 1);
}

#[test]
fn the_probe_address_follows_the_configured_environment() {
    let mut service = ServiceEntry::new("nas", "NAS", "http://nas.local");
    service
        .addresses
        .insert("local".into(), "http://192.168.1.10".into());
    service
        .addresses
        .insert("internet".into(), "https://nas.example.com".into());
    let local = Environment::parse("local").unwrap();
    assert_eq!(service.probe_address(&local), "http://192.168.1.10");
    assert_eq!(
        service.probe_address(&Environment::internet()),
        "https://nas.example.com"
    );
    service.probe.environment = Some("internet".into());
    assert_eq!(service.probe_address(&local), "https://nas.example.com");
    let bare = ServiceEntry::new("printer", "Printer", "http://printer.local");
    assert_eq!(bare.probe_address(&local), "http://printer.local");
}

#[test]
fn a_tcp_service_may_use_any_scheme_with_a_host() {
    let mut ssh = ServiceEntry::new("nas-ssh", "NAS SSH", "ssh://nas.home.lan");
    ssh.probe.kind = ProbeKind::Tcp;
    assert!(check_entry(&ssh, &Known::default()).is_empty());
    let mut ping = ServiceEntry::new("printer", "Printer", "icmp://printer.home.lan");
    ping.probe.kind = ProbeKind::Icmp;
    assert!(check_entry(&ping, &Known::default()).is_empty());
}

#[test]
fn a_tcp_service_without_any_port_is_refused_on_probe_port() {
    let mut printer = ServiceEntry::new("printer", "Printer", "tcp://printer.home.lan");
    printer.probe.kind = ProbeKind::Tcp;
    let errors = check_entry(&printer, &Known::default());
    assert_eq!(fields(errors.clone()), vec!["probe.port"]);
    printer.probe.port = Some(9100);
    assert!(check_entry(&printer, &Known::default()).is_empty());
}

#[test]
fn an_http_service_still_needs_an_http_address() {
    let mut ssh = ServiceEntry::new("nas-ssh", "NAS SSH", "ssh://nas.home.lan");
    ssh.probe.kind = ProbeKind::Http;
    assert_eq!(fields(check_entry(&ssh, &Known::default())), vec!["url"]);
}

#[test]
fn an_unknown_probe_kind_is_refused_by_the_section() {
    let document: DocumentMut =
        "[[services]]\nid = \"a\"\nname = \"A\"\nurl = \"http://a\"\nprobe = { kind = \"udp\" }\n"
            .parse()
            .unwrap();
    let errors = validate_services(&document);
    assert_eq!(fields(errors.clone()), vec!["services"]);
    assert!(errors[0].message.contains("udp"));
}

#[tokio::test]
async fn a_wake_probes_a_service_in_backoff_at_once_and_resets_the_backoff() {
    let failing = Upstream::start(Behaviour::Status {
        code: 500,
        delay: Duration::ZERO,
    })
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    write(&path, &service_text("media", &failing.url()));
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let board = Arc::new(StatusBoard::watched(OffsetDateTime::now_utc(), Vec::new()));
    let supervisor = Supervisor::new(
        store,
        Environment::internet(),
        board.clone(),
        Arc::new(Probe::new().unwrap()),
    );
    supervisor.reconcile();
    assert!(eventually(|| failing.hits() == 1).await);
    assert_eq!(supervisor.wake("media"), Wake::Woken);
    assert!(eventually(|| failing.hits() == 2).await);
    assert!(
        matches!(supervisor.wake("media"), Wake::TooSoon(wait) if wait <= Supervisor::WAKE_INTERVAL)
    );
    assert_eq!(supervisor.wake("missing"), Wake::NotFound);
}

#[tokio::test]
async fn a_service_with_probing_disabled_cannot_be_woken() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    write(
        &path,
        "[[services]]\nid = \"printer\"\nname = \"Printer\"\nurl = \"http://10.255.0.3\"\nprobe = { enabled = false }\n",
    );
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let board = Arc::new(StatusBoard::watched(OffsetDateTime::now_utc(), Vec::new()));
    let supervisor = Supervisor::new(
        store,
        Environment::internet(),
        board,
        Arc::new(Probe::new().unwrap()),
    );
    assert_eq!(supervisor.wake("printer"), Wake::Disabled);
}

#[test]
fn links_notes_and_related_widgets_are_checked_field_by_field() {
    let mut media = entry();
    media.links = vec![
        crate::types::ServiceLink {
            title: String::new(),
            url: "ftp://x".into(),
        },
        crate::types::ServiceLink {
            title: "Docs".into(),
            url: "https://jellyfin.org/docs".into(),
        },
    ];
    media.notes = Some("x".repeat(10_001));
    media.widgets = vec!["nas-disks".into(), "missing".into()];
    let known = Known {
        widgets: vec!["nas-disks".into()],
        ..Known::default()
    };
    let errors = check_entry(&media, &known);
    assert_eq!(
        fields(errors.clone()),
        vec!["links[0].title", "links[0].url", "notes", "widgets[1]"]
    );
    assert!(errors[3].message.contains("missing"));
}

#[test]
fn a_related_widget_is_resolved_against_the_layout_in_the_file() {
    let text = "[[dashboard.widgets]]\ntype = \"host-metrics\"\nid = \"nas-disks\"\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://nas\"\nwidgets = [\"nas-disks\", \"missing\"]\n";
    let document: DocumentMut = text.parse().unwrap();
    assert_eq!(
        fields(validate_services(&document)),
        vec!["services[0].widgets[1]"]
    );
}

#[derive(Default)]
struct Recorder {
    changes: Mutex<Vec<StatusChange>>,
}

impl StatusObserver for Recorder {
    fn changed(&self, change: &StatusChange) {
        self.changes.lock().unwrap().push(change.clone());
    }
}

#[test]
fn a_service_that_does_not_notify_still_reports_its_changes_marked_as_silent() {
    let recorder = Arc::new(Recorder::default());
    let board = StatusBoard::watched(OffsetDateTime::now_utc(), vec![recorder.clone()]);
    let mut silent = entry();
    silent.notify = Some(false);
    let now = OffsetDateTime::now_utc();
    board.record(&silent, ProbeOutcome::answered(ServiceState::Up, 5), now);
    board.record(
        &silent,
        ProbeOutcome::failed(ServiceState::Down, None, "refused".into()),
        now,
    );
    let changes = recorder.changes.lock().unwrap();
    assert_eq!(changes.len(), 2);
    assert!(changes.iter().all(|change| !change.notify));
    assert_eq!(changes[1].now, "down");
    assert_eq!(changes[1].diagnosis.as_deref(), Some("other"));
}
