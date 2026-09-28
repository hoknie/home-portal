use std::sync::Mutex;

use async_trait::async_trait;

use crate::ports::PortalActions;
use crate::types::{ProbeResult, StatusResult};

#[derive(Default)]
pub struct FakeActions {
    pub sent: Mutex<Vec<String>>,
    pub states: Mutex<Vec<String>>,
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
        Ok(ProbeResult {
            state,
            latency_milliseconds: Some(12),
            diagnosis: None,
        })
    }

    async fn status(&self, service: &str) -> Result<StatusResult, String> {
        Ok(StatusResult {
            state: if service == "nas" {
                "down".into()
            } else {
                "up".into()
            },
            since: None,
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
