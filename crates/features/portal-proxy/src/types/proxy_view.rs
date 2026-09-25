use serde_json::Value;

use super::{ProxySettings, PublishedService, SyncState};
use crate::responses::CaddyResponse;

#[derive(Debug, Clone)]
pub struct ProxyView {
    pub settings: ProxySettings,
    pub services: Vec<PublishedService>,
    pub rendered: Option<Value>,
    pub state: SyncState,
    pub portal: String,
    pub caddy: CaddyResponse,
}
