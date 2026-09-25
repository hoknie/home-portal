use portal_model::Environment;

use crate::helpers::{probe_host, probe_target, tcp_port};
use crate::probes::Probe;
use crate::types::{ProbeKind, ProbeReport, ServiceEntry};

pub async fn probe_once(entry: &ServiceEntry, host: &Environment) -> Result<ProbeReport, String> {
    let probe = Probe::new()?;
    let address = entry.probe_address(host).to_string();
    let target = match entry.probe.kind {
        ProbeKind::Http => probe_target(&address, &entry.probe.path)
            .map(|url| url.to_string())
            .unwrap_or_else(|_| address.clone()),
        ProbeKind::Tcp => match (probe_host(&address), tcp_port(&address, entry.probe.port)) {
            (Ok(name), Ok(port)) => format!("{name}:{port}"),
            _ => address.clone(),
        },
        ProbeKind::Icmp => probe_host(&address).unwrap_or_else(|_| address.clone()),
    };
    let outcome = probe.run(entry, &address).await;
    Ok(ProbeReport {
        kind: entry.probe.kind,
        target,
        outcome,
    })
}
