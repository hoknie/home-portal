use std::process::ExitCode;

use portal_config::{ConfigStore, configuration_path};
use portal_model::Environment;
use portal_network::{host_environment, read_environments};
use portal_services::{ProbeKind, ProbeReport, ServiceEntry, ServicesSection, advice, probe_once};

pub const PROBE: &str = "probe";
pub const KIND_FLAG: &str = "--kind";
pub const USAGE: &str = "usage: home-portal probe <service id | url> [--kind http|tcp|icmp]";
pub const ADHOC_ID: &str = "probe";

pub async fn probe(arguments: &[String]) -> ExitCode {
    let (entry, host) = match target_of(arguments) {
        Ok(target) => target,
        Err(message) => {
            eprintln!("home-portal: {message}");
            return ExitCode::FAILURE;
        }
    };
    match probe_once(&entry, &host).await {
        Ok(report) => {
            print!("{}", describe(&entry, &report));
            if report.outcome.state.answered() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("home-portal: cannot probe: {message}");
            ExitCode::FAILURE
        }
    }
}

fn target_of(arguments: &[String]) -> Result<(ServiceEntry, Environment), String> {
    let (target, kind) = match arguments {
        [target] => (target, None),
        [target, flag, kind] if flag == KIND_FLAG => (target, Some(kind_of(kind)?)),
        _ => return Err(USAGE.to_string()),
    };
    if target.contains("://") {
        let mut entry = ServiceEntry::new(ADHOC_ID, target, target);
        entry.probe.kind = kind.unwrap_or_default();
        return Ok((entry, Environment::internet()));
    }
    let store = ConfigStore::open(configuration_path()).map_err(|error| error.to_string())?;
    let document = store.read().document;
    let mut entry = ServicesSection::read(&document)?
        .services
        .into_iter()
        .find(|entry| &entry.id == target)
        .ok_or_else(|| format!("no service {target:?} in {}", store.path().display()))?;
    if let Some(kind) = kind {
        entry.probe.kind = kind;
    }
    let host = host_environment(&read_environments(&document).unwrap_or_default());
    Ok((entry, host))
}

fn kind_of(name: &str) -> Result<ProbeKind, String> {
    match name {
        "http" => Ok(ProbeKind::Http),
        "tcp" => Ok(ProbeKind::Tcp),
        "icmp" => Ok(ProbeKind::Icmp),
        other => Err(format!("unknown probe kind {other:?}; {USAGE}")),
    }
}

fn describe(entry: &ServiceEntry, report: &ProbeReport) -> String {
    let outcome = &report.outcome;
    let mut lines = vec![
        format!("service    {}", entry.id),
        format!("kind       {}", report.kind.name()),
        format!("target     {}", report.target),
        format!("state      {}", outcome.state.name()),
    ];
    if let Some(latency) = outcome.latency_milliseconds {
        lines.push(format!("latency    {latency} ms"));
    }
    if let Some(error) = &outcome.error {
        lines.push(format!("error      {error}"));
    }
    if let Some(diagnosis) = outcome.diagnosis {
        lines.push(format!("diagnosis  {}", diagnosis.code()));
        lines.push(format!("advice     {}", advice(diagnosis)));
    }
    lines.push(String::new());
    lines.join("\n")
}
