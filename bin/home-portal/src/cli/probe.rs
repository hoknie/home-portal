use std::io::Write;
use std::process::ExitCode;
use std::sync::Arc;

use clap::builder::styling::Style;
use portal_config::{ConfigStore, configuration_path};
use portal_model::{Environment, ServiceState};
use portal_network::{CurrentEnvironments, host_environment};
use portal_services::{ProbeKind, ProbeReport, ServiceEntries, ServiceEntry, advice, probe_once};

use super::failure::fail;
use super::palette::{ALERT, EMPHASIS, ERROR, MUTED, SUCCESS, WARNING};

pub const ADHOC_ID: &str = "probe";
pub const LABEL_WIDTH: usize = 10;

pub async fn probe(target: String, kind: Option<ProbeKind>) -> ExitCode {
    let (entry, host) = match target_of(&target, kind) {
        Ok(target) => target,
        Err(message) => return fail(message),
    };
    match probe_once(&entry, &host).await {
        Ok(report) => {
            let _ = write!(anstream::stdout(), "{}", describe(&entry, &report));
            if report.outcome.state.answered() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => fail(format!("cannot probe: {message}")),
    }
}

fn target_of(target: &str, kind: Option<ProbeKind>) -> Result<(ServiceEntry, Environment), String> {
    if target.contains("://") {
        let mut entry = ServiceEntry::new(ADHOC_ID, target, target);
        entry.probe.kind = kind.unwrap_or_default();
        return Ok((entry, Environment::internet()));
    }
    let store = Arc::new(
        configuration_path()
            .and_then(|location| ConfigStore::open_located(&location))
            .map_err(|error| error.to_string())?,
    );
    let mut entry = ServiceEntries::new(store.clone())
        .run()?
        .into_iter()
        .find(|entry| entry.id == target)
        .ok_or_else(|| format!("no service {target:?} in {}", store.path().display()))?;
    if let Some(kind) = kind {
        entry.probe.kind = kind;
    }
    let host = host_environment(
        &CurrentEnvironments::new(store.clone())
            .run()
            .unwrap_or_default(),
    );
    Ok((entry, host))
}

fn describe(entry: &ServiceEntry, report: &ProbeReport) -> String {
    let outcome = &report.outcome;
    let state = state_style(outcome.state);
    let mut lines = vec![
        line("service", &entry.id),
        line("kind", report.kind.name()),
        line("target", &report.target),
        line(
            "state",
            &format!("{state}{}{state:#}", outcome.state.name()),
        ),
    ];
    if let Some(latency) = outcome.latency_milliseconds {
        lines.push(line("latency", &format!("{latency} ms")));
    }
    if let Some(error) = &outcome.error {
        lines.push(line("error", error));
    }
    if let Some(diagnosis) = outcome.diagnosis {
        lines.push(line(
            "diagnosis",
            &format!("{EMPHASIS}{}{EMPHASIS:#}", diagnosis.code()),
        ));
        lines.push(line("advice", advice(diagnosis)));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn line(label: &str, value: &str) -> String {
    format!("{MUTED}{label:<LABEL_WIDTH$}{MUTED:#} {value}")
}

fn state_style(state: ServiceState) -> Style {
    match state {
        ServiceState::Up => SUCCESS,
        ServiceState::Degraded => WARNING,
        ServiceState::Down => ERROR,
        ServiceState::Unreadable => ALERT,
        ServiceState::Unknown => MUTED,
    }
}
