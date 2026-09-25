use std::time::Duration;

use std::time::Instant;

use portal_model::{Diagnosis, ProbeOutcome, ServiceState};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use super::{HttpProbe, Probe};
use crate::fakes::{Behaviour, Upstream};
use crate::types::{ProbeKind, ProbeSettings, ServiceEntry};

const FAST: Duration = Duration::from_millis(0);

fn entry(url: &str) -> ServiceEntry {
    let mut entry = ServiceEntry::new("service", "Service", url);
    entry.probe = ProbeSettings {
        timeout_seconds: 1,
        degraded_after_milliseconds: 300,
        ..ProbeSettings::default()
    };
    entry
}

async fn state_of(url: &str) -> ServiceState {
    HttpProbe::new()
        .unwrap()
        .probe(&entry(url), url)
        .await
        .state
}

async fn free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn a_fast_success_is_up() {
    let upstream = Upstream::start(Behaviour::Status {
        code: 200,
        delay: FAST,
    })
    .await;
    let outcome = HttpProbe::new()
        .unwrap()
        .probe(&entry(&upstream.url()), &upstream.url())
        .await;
    assert_eq!(outcome.state, ServiceState::Up);
    assert!(outcome.latency_milliseconds.is_some());
    assert!(outcome.error.is_none());
}

#[tokio::test]
async fn a_slow_success_is_degraded() {
    let upstream = Upstream::start(Behaviour::Status {
        code: 200,
        delay: Duration::from_millis(500),
    })
    .await;
    assert_eq!(state_of(&upstream.url()).await, ServiceState::Degraded);
}

#[tokio::test]
async fn a_server_error_is_down() {
    let upstream = Upstream::start(Behaviour::Status {
        code: 500,
        delay: FAST,
    })
    .await;
    let outcome = HttpProbe::new()
        .unwrap()
        .probe(&entry(&upstream.url()), &upstream.url())
        .await;
    assert_eq!(outcome.state, ServiceState::Down);
    assert!(outcome.error.unwrap().contains("500"));
}

#[tokio::test]
async fn a_refused_connection_is_down() {
    let port = free_port().await;
    assert_eq!(
        state_of(&format!("http://127.0.0.1:{port}")).await,
        ServiceState::Down
    );
}

#[tokio::test]
async fn a_server_that_accepts_and_never_answers_is_down_after_the_timeout() {
    let upstream = Upstream::start(Behaviour::Hang).await;
    assert_eq!(state_of(&upstream.url()).await, ServiceState::Down);
}

#[tokio::test]
async fn a_name_that_does_not_resolve_is_unreadable_not_down() {
    assert_eq!(
        state_of("http://no-such-host.invalid").await,
        ServiceState::Unreadable
    );
}

#[tokio::test]
async fn an_answer_that_is_not_http_is_unreadable() {
    let upstream = Upstream::start(Behaviour::Garbage).await;
    assert_eq!(state_of(&upstream.url()).await, ServiceState::Unreadable);
}

#[tokio::test]
async fn a_redirect_is_an_answer_and_is_not_followed() {
    let upstream = Upstream::start(Behaviour::Redirect).await;
    assert_eq!(state_of(&upstream.url()).await, ServiceState::Up);
    assert_eq!(upstream.hits(), 1);
}

async fn outcome_of(url: &str) -> ProbeOutcome {
    Probe::new().unwrap().run(&entry(url), url).await
}

fn tcp_entry(url: &str, port: Option<u16>) -> ServiceEntry {
    let mut tcp = entry(url);
    tcp.probe.kind = ProbeKind::Tcp;
    tcp.probe.port = port;
    tcp
}

#[tokio::test]
async fn every_failure_names_its_diagnosis() {
    let error = Upstream::start(Behaviour::Status {
        code: 503,
        delay: FAST,
    })
    .await;
    let hang = Upstream::start(Behaviour::Hang).await;
    let garbage = Upstream::start(Behaviour::Garbage).await;
    let plain = Upstream::start(Behaviour::Status {
        code: 200,
        delay: FAST,
    })
    .await;
    let port = free_port().await;
    let cases = [
        (error.url(), ServiceState::Down, Diagnosis::HttpStatus),
        (hang.url(), ServiceState::Down, Diagnosis::Timeout),
        (garbage.url(), ServiceState::Unreadable, Diagnosis::NotHttp),
        (
            format!("https://{}", plain.address),
            ServiceState::Unreadable,
            Diagnosis::Tls,
        ),
        (
            format!("http://127.0.0.1:{port}"),
            ServiceState::Down,
            Diagnosis::Refused,
        ),
        (
            "http://no-such-host.invalid".to_string(),
            ServiceState::Unreadable,
            Diagnosis::NameNotResolved,
        ),
    ];
    for (url, state, diagnosis) in cases {
        let outcome = outcome_of(&url).await;
        assert_eq!(
            (outcome.state, outcome.diagnosis),
            (state, Some(diagnosis)),
            "{url}: {:?}",
            outcome.error
        );
    }
}

#[tokio::test]
async fn a_success_carries_no_diagnosis() {
    let upstream = Upstream::start(Behaviour::Status {
        code: 200,
        delay: FAST,
    })
    .await;
    assert_eq!(outcome_of(&upstream.url()).await.diagnosis, None);
}

#[tokio::test]
async fn a_tcp_probe_connects_sends_nothing_and_is_up() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let received = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = Vec::new();
        socket.read_to_end(&mut buffer).await.unwrap();
        buffer.len()
    });
    let entry = tcp_entry("ssh://127.0.0.1", Some(port));
    let outcome = Probe::new().unwrap().run(&entry, &entry.url).await;
    assert_eq!(outcome.state, ServiceState::Up);
    assert_eq!(received.await.unwrap(), 0);
}

#[tokio::test]
async fn a_tcp_probe_of_a_closed_port_is_down_and_refused() {
    let port = free_port().await;
    let entry = tcp_entry(&format!("tcp://127.0.0.1:{port}"), None);
    let outcome = Probe::new().unwrap().run(&entry, &entry.url).await;
    assert_eq!(
        (outcome.state, outcome.diagnosis),
        (ServiceState::Down, Some(Diagnosis::Refused))
    );
}

#[tokio::test]
async fn a_tcp_probe_that_gets_no_answer_is_down_within_its_timeout() {
    let entry = tcp_entry("tcp://10.255.255.1:9", None);
    let started = Instant::now();
    let outcome = Probe::new().unwrap().run(&entry, &entry.url).await;
    assert!(started.elapsed() < std::time::Duration::from_millis(1500));
    assert!(outcome.state.failed());
    assert!(
        matches!(
            outcome.diagnosis,
            Some(Diagnosis::Timeout | Diagnosis::HostUnreachable | Diagnosis::LocalNetworkDenied)
        ),
        "{:?}",
        outcome
    );
}

#[tokio::test]
async fn an_icmp_probe_of_loopback_answers_or_says_it_may_not_ping() {
    let mut icmp = entry("icmp://127.0.0.1");
    icmp.probe.kind = ProbeKind::Icmp;
    let outcome = Probe::new().unwrap().run(&icmp, &icmp.url).await;
    match outcome.state {
        ServiceState::Up | ServiceState::Degraded => assert_eq!(outcome.diagnosis, None),
        _ => assert_eq!(
            outcome.diagnosis,
            Some(Diagnosis::IcmpNotPermitted),
            "{:?}",
            outcome.error
        ),
    }
}
