use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use time::OffsetDateTime;

use crate::types::Delivery;

#[derive(Default)]
pub struct DeliveryBook {
    last: Mutex<BTreeMap<String, Delivery>>,
    errors: Mutex<BTreeMap<String, (OffsetDateTime, String)>>,
}

impl DeliveryBook {
    pub fn record(&self, delivery: Delivery) {
        if let Some(error) = &delivery.error {
            self.errors
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(delivery.channel.clone(), (delivery.at, error.clone()));
        }
        self.last
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(delivery.channel.clone(), delivery);
    }

    pub fn last(&self, channel: &str) -> Option<Delivery> {
        self.last
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(channel)
            .cloned()
    }

    pub fn last_error(&self, channel: &str) -> Option<(OffsetDateTime, String)> {
        self.errors
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(channel)
            .cloned()
    }
}
