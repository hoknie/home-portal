use portal_feature::ChannelReadiness;
use serde_json::Value;
use time::OffsetDateTime;

use super::Delivery;

#[derive(Debug, Clone)]
pub struct ChannelView {
    pub name: String,
    pub readiness: ChannelReadiness,
    pub settings: Value,
    pub last: Option<Delivery>,
    pub last_error: Option<(OffsetDateTime, String)>,
    pub queued: usize,
    pub dropped: usize,
}
