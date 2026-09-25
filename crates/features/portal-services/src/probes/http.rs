use std::time::{Duration, Instant};

use portal_model::{Diagnosis, ProbeOutcome, ServiceState};
use reqwest::Client;
use reqwest::redirect::Policy;

use crate::helpers::{classify_failure, probe_target};
use crate::types::ServiceEntry;

pub struct HttpProbe {
    client: Client,
}

impl HttpProbe {
    pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
    pub const BODY_LIMIT_BYTES: usize = 64 * 1024;
    pub const USER_AGENT: &'static str = "home-portal";

    pub fn new() -> Result<HttpProbe, String> {
        Client::builder()
            .connect_timeout(Self::CONNECT_TIMEOUT)
            .redirect(Policy::none())
            .user_agent(Self::USER_AGENT)
            .build()
            .map(|client| HttpProbe { client })
            .map_err(|error| error.to_string())
    }

    pub async fn probe(&self, entry: &ServiceEntry, address: &str) -> ProbeOutcome {
        let target = match probe_target(address, &entry.probe.path) {
            Ok(target) => target,
            Err(message) => {
                return ProbeOutcome::failed(ServiceState::Unreadable, None, message);
            }
        };
        let started = Instant::now();
        let sent = self
            .client
            .get(target)
            .timeout(Duration::from_secs(entry.probe.timeout_seconds))
            .send()
            .await;
        let mut response = match sent {
            Ok(response) => response,
            Err(error) => return classify_failure(&error).into_outcome(None),
        };
        let latency = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let mut read = 0usize;
        while read < Self::BODY_LIMIT_BYTES {
            match response.chunk().await {
                Ok(Some(chunk)) => read += chunk.len(),
                Ok(None) => break,
                Err(error) => return classify_failure(&error).into_outcome(Some(latency)),
            }
        }
        let status = response.status();
        if status.is_success() || status.is_redirection() {
            let state = if latency > entry.probe.degraded_after_milliseconds {
                ServiceState::Degraded
            } else {
                ServiceState::Up
            };
            return ProbeOutcome::answered(state, latency);
        }
        ProbeOutcome::failed(
            ServiceState::Down,
            Some(latency),
            format!("answered {status}"),
        )
        .because(Diagnosis::HttpStatus)
    }
}
