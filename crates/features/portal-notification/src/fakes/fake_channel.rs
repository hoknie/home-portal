use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use portal_feature::{Channel, ChannelReadiness, FieldError, Notification, SecretSource};
use serde_json::{Value, json};
use toml_edit::{DocumentMut, Table};

pub struct FakeChannel {
    pub name: &'static str,
    pub ready: bool,
    pub delay: Duration,
    pub failing: bool,
    pub problems: Vec<FieldError>,
    pub sent: Mutex<Vec<String>>,
}

impl FakeChannel {
    pub fn ready(name: &'static str) -> FakeChannel {
        FakeChannel {
            name,
            ready: true,
            delay: Duration::ZERO,
            failing: false,
            problems: Vec::new(),
            sent: Mutex::new(Vec::new()),
        }
    }

    pub fn sent(&self) -> Vec<String> {
        self.sent
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl Channel for FakeChannel {
    fn name(&self) -> &'static str {
        self.name
    }

    fn problems(&self, _: &DocumentMut, _: &dyn SecretSource) -> Vec<FieldError> {
        self.problems.clone()
    }

    fn readiness(&self, _: &DocumentMut, _: &dyn SecretSource) -> ChannelReadiness {
        if self.ready {
            ChannelReadiness::Ready
        } else {
            ChannelReadiness::Disabled
        }
    }

    fn settings(&self, _: &DocumentMut) -> Value {
        json!({ "enabled": self.ready })
    }

    fn apply(&self, _: &mut Table, _: &Value) -> Result<(), Vec<FieldError>> {
        Ok(())
    }

    async fn deliver(
        &self,
        notification: &Notification,
        _: &DocumentMut,
        _: &dyn SecretSource,
    ) -> Result<(), String> {
        tokio::time::sleep(self.delay).await;
        if self.failing {
            return Err("the channel is down".into());
        }
        self.sent
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(notification.message());
        Ok(())
    }
}
