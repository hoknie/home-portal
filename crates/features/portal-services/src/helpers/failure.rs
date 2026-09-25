use std::error::Error;
use std::io::{self, ErrorKind};

use portal_model::{Diagnosis, ServiceState};

use crate::types::Failure;

pub const DNS_MARKERS: [&str; 3] = [
    "dns error",
    "failed to lookup address",
    "nodename nor servname",
];

pub fn classify_failure(error: &reqwest::Error) -> Failure {
    if error.is_timeout() {
        return Failure::new(ServiceState::Down, Diagnosis::Timeout, "timed out");
    }
    let mut source: Option<&(dyn Error + 'static)> = error.source();
    let mut deepest = error.to_string();
    while let Some(current) = source {
        if let Some(found) = classify_source(current) {
            return found;
        }
        deepest = current.to_string();
        source = current.source();
    }
    Failure::new(ServiceState::Unreadable, Diagnosis::Other, deepest)
}

pub fn classify_io(error: &io::Error) -> Failure {
    let message = error.to_string();
    match error.kind() {
        ErrorKind::ConnectionRefused => {
            Failure::new(ServiceState::Down, Diagnosis::Refused, message)
        }
        ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
            Failure::new(ServiceState::Down, Diagnosis::Refused, message)
        }
        ErrorKind::TimedOut => Failure::new(ServiceState::Down, Diagnosis::Timeout, message),
        ErrorKind::HostUnreachable | ErrorKind::NetworkUnreachable => {
            Failure::new(ServiceState::Down, Diagnosis::HostUnreachable, message)
        }
        ErrorKind::PermissionDenied => {
            Failure::new(ServiceState::Unreadable, Diagnosis::Other, message)
        }
        _ if names_dns(&message) => Failure::new(
            ServiceState::Unreadable,
            Diagnosis::NameNotResolved,
            message,
        ),
        _ => Failure::new(ServiceState::Unreadable, Diagnosis::Other, message),
    }
}

fn classify_source(current: &(dyn Error + 'static)) -> Option<Failure> {
    if let Some(tls) = current.downcast_ref::<rustls::Error>() {
        return Some(Failure::new(
            ServiceState::Unreadable,
            Diagnosis::Tls,
            tls.to_string(),
        ));
    }
    if let Some(hyper) = current.downcast_ref::<hyper::Error>()
        && hyper.is_parse()
    {
        return Some(Failure::new(
            ServiceState::Unreadable,
            Diagnosis::NotHttp,
            hyper.to_string(),
        ));
    }
    if let Some(io) = current.downcast_ref::<io::Error>() {
        if let Some(tls) = wrapped_tls(io) {
            return Some(Failure::new(
                ServiceState::Unreadable,
                Diagnosis::Tls,
                tls.to_string(),
            ));
        }
        let failure = classify_io(io);
        if failure.diagnosis != Diagnosis::Other {
            return Some(failure);
        }
    }
    if names_dns(&current.to_string()) {
        return Some(Failure::new(
            ServiceState::Unreadable,
            Diagnosis::NameNotResolved,
            current.to_string(),
        ));
    }
    None
}

fn wrapped_tls(error: &io::Error) -> Option<&rustls::Error> {
    let mut current = error.get_ref()?;
    loop {
        if let Some(tls) = current.downcast_ref::<rustls::Error>() {
            return Some(tls);
        }
        current = current.downcast_ref::<io::Error>()?.get_ref()?;
    }
}

fn names_dns(message: &str) -> bool {
    let lower = message.to_lowercase();
    DNS_MARKERS.iter().any(|marker| lower.contains(marker))
}
