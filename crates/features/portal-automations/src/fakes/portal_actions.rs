use std::sync::Mutex;

use async_trait::async_trait;

use crate::ports::PortalActions;
use crate::types::{PortalService, PortalState, ProbeResult, StatusResult};

#[derive(Default)]
pub struct FakeActions {
    pub sent: Mutex<Vec<String>>,
    pub states: Mutex<Vec<String>>,
    pub probed: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl PortalActions for FakeActions {
    async fn probe(&self, service: &str) -> Result<ProbeResult, String> {
        let state = self
            .states
            .lock()
            .unwrap()
            .pop()
            .unwrap_or_else(|| "up".into());
        if service == "ghost" {
            return Err("no service ghost".into());
        }
        self.probed
            .lock()
            .unwrap()
            .push((service.to_string(), state.clone()));
        Ok(ProbeResult {
            state,
            latency_milliseconds: Some(12),
            diagnosis: None,
        })
    }

    async fn status(&self, service: &str) -> Result<StatusResult, String> {
        let state = match service {
            "nas" => "down",
            "media" | "jellyfin" | "router" => "up",
            other => return Err(format!("no service {other}")),
        };
        Ok(StatusResult {
            state: state.into(),
            since: None,
        })
    }

    async fn state(&self) -> Result<PortalState, String> {
        let probed = self.probed.lock().unwrap();
        let state_of = |id: &str| {
            probed
                .iter()
                .rev()
                .find(|(service, _)| service == id)
                .map_or_else(|| "down".to_string(), |(_, state)| state.clone())
        };
        let service = |id: &str, name: &str| PortalService {
            id: id.into(),
            name: name.into(),
            group: Some("Home".into()),
            url: format!("http://{id}.lan"),
            address: format!("http://{id}.lan"),
            state: state_of(id),
            since: Some("2026-09-28T09:00:00Z".into()),
            latency_milliseconds: Some(12),
            public: id == "media",
        };
        Ok(PortalState {
            services: vec![service("media", "Media"), service("nas", "NAS")],
            address: "0.0.0.0".into(),
            port: 8080,
            url: "http://portal.lan:8080".into(),
            environments: vec!["local".into(), "vpn".into()],
        })
    }

    async fn notify(
        &self,
        channel: Option<&str>,
        _title: &str,
        text: &str,
    ) -> Result<String, String> {
        if channel == Some("sms") {
            return Err("there is no notification channel sms".into());
        }
        self.sent.lock().unwrap().push(text.to_string());
        Ok(format!("delivered: {}", channel.unwrap_or("telegram")))
    }
}
