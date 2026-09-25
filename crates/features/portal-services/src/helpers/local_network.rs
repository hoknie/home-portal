use std::time::Duration;

use ipnet::IpNet;
use portal_model::{Diagnosis, ProbeOutcome, ServiceState};

use crate::types::Attempt;

pub const INSTANT_REFUSAL: Duration = Duration::from_millis(100);
pub const PLATFORM_REFUSES_LOCAL_NETWORK: bool = cfg!(target_os = "macos");
pub const LOCAL_NETWORK_DENIED: &str = "the operating system refused access to the local network (no route to host at once, although the address is on a directly connected network)";

pub fn local_network_denied(attempt: &Attempt, subnets: &[IpNet], platform_applies: bool) -> bool {
    let Some(target) = attempt.target else {
        return false;
    };
    platform_applies
        && attempt.elapsed < INSTANT_REFUSAL
        && subnets
            .iter()
            .any(|subnet| subnet.prefix_len() < subnet.max_prefix_len() && subnet.contains(&target))
}

pub fn refine_unreachable(outcome: ProbeOutcome, denied: bool) -> ProbeOutcome {
    if outcome.diagnosis != Some(Diagnosis::HostUnreachable) || !denied {
        return outcome;
    }
    let message = outcome.error.unwrap_or_default();
    ProbeOutcome::failed(
        ServiceState::Unreadable,
        outcome.latency_milliseconds,
        format!("{LOCAL_NETWORK_DENIED}: {message}"),
    )
    .because(Diagnosis::LocalNetworkDenied)
}
