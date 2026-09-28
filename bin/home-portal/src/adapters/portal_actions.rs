use std::sync::OnceLock;

use async_trait::async_trait;
use portal_automations::{PortalActions, PortalService, PortalState, ProbeResult, StatusResult};
use portal_feature::Notification;
use portal_feature::PortalEvent;
use portal_model::Environment;
use portal_network::CurrentNetwork;
use portal_notification::SendNotification;
use portal_services::{CurrentStatus, ProbeService, ServiceEntries};

#[derive(Default)]
pub struct WorkflowActions {
    pub probe: OnceLock<ProbeService>,
    pub status: OnceLock<CurrentStatus>,
    pub notify: OnceLock<SendNotification>,
    pub entries: OnceLock<ServiceEntries>,
    pub network: OnceLock<CurrentNetwork>,
    pub host: OnceLock<Environment>,
}

impl WorkflowActions {
    pub const NOT_READY: &'static str = "the portal is still starting";
    pub const DELIVERED: &'static str = "delivered through";
    pub const FAILED: &'static str = "not delivered through";
}

#[async_trait]
impl PortalActions for WorkflowActions {
    async fn probe(&self, service: &str) -> Result<ProbeResult, String> {
        let probe = self
            .probe
            .get()
            .ok_or_else(|| Self::NOT_READY.to_string())?;
        let outcome = probe.run(service).await?;
        Ok(ProbeResult {
            state: outcome.state.name().to_string(),
            latency_milliseconds: outcome.latency_milliseconds,
            diagnosis: outcome
                .diagnosis
                .map(|diagnosis| diagnosis.code().to_string()),
        })
    }

    async fn status(&self, service: &str) -> Result<StatusResult, String> {
        let current = self
            .status
            .get()
            .ok_or_else(|| Self::NOT_READY.to_string())?;
        let status = current.run(service)?;
        Ok(StatusResult {
            state: status.state.name().to_string(),
            since: Some(PortalEvent::timestamp(status.since)),
        })
    }

    async fn state(&self) -> Result<PortalState, String> {
        let not_ready = || Self::NOT_READY.to_string();
        let entries = self.entries.get().ok_or_else(not_ready)?.run()?;
        let current = self.status.get().ok_or_else(not_ready)?;
        let host = self.host.get().ok_or_else(not_ready)?;
        let reading = self.network.get().ok_or_else(not_ready)?.run();
        let settings = reading.settings.unwrap_or_default();
        let services = entries
            .into_iter()
            .map(|entry| {
                let status = current.run(&entry.id).ok();
                PortalService {
                    address: entry
                        .addresses
                        .get(host.as_str())
                        .cloned()
                        .unwrap_or_else(|| entry.url.clone()),
                    state: status
                        .as_ref()
                        .map_or("unknown", |status| status.state.name())
                        .to_string(),
                    since: status
                        .as_ref()
                        .map(|status| PortalEvent::timestamp(status.since)),
                    latency_milliseconds: status.and_then(|status| status.latency_milliseconds),
                    id: entry.id,
                    name: entry.name,
                    group: entry.group,
                    url: entry.url,
                    public: entry.public,
                }
            })
            .collect();
        let url = settings
            .public_url
            .clone()
            .unwrap_or_else(|| format!("http://{}", settings.socket_address()));
        Ok(PortalState {
            services,
            address: settings.address.to_string(),
            port: settings.port,
            url,
            environments: reading
                .environments
                .map(|environments| {
                    environments
                        .names()
                        .iter()
                        .map(|name| name.as_str().to_string())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    async fn notify(
        &self,
        channel: Option<&str>,
        title: &str,
        text: &str,
    ) -> Result<String, String> {
        let deliveries = self
            .notify
            .get()
            .ok_or_else(|| Self::NOT_READY.to_string())?
            .run(channel, &Notification::new(title, text))
            .await?;
        let delivered: Vec<&str> = deliveries
            .iter()
            .filter(|delivery| delivery.succeeded())
            .map(|delivery| delivery.channel.as_str())
            .collect();
        let failed: Vec<String> = deliveries
            .iter()
            .filter_map(|delivery| {
                delivery
                    .error
                    .as_ref()
                    .map(|error| format!("{}: {error}", delivery.channel))
            })
            .collect();
        let detail = [
            (!delivered.is_empty())
                .then(|| format!("{} {}", Self::DELIVERED, delivered.join(", "))),
            (!failed.is_empty()).then(|| format!("{} {}", Self::FAILED, failed.join("; "))),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("; ");
        if delivered.is_empty() {
            Err(detail)
        } else {
            Ok(detail)
        }
    }
}
