use portal_feature::PortalEvent;
use schemars::JsonSchema;
use serde::Serialize;

use crate::types::Delivery;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DeliveryResponse {
    pub channel: String,
    pub at: String,
    pub delivered: bool,
    pub error: Option<String>,
}

impl DeliveryResponse {
    pub fn of(delivery: &Delivery) -> DeliveryResponse {
        DeliveryResponse {
            channel: delivery.channel.clone(),
            at: PortalEvent::timestamp(delivery.at),
            delivered: delivery.succeeded(),
            error: delivery.error.clone(),
        }
    }
}
