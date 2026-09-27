use std::sync::Arc;

use crate::types::{DnsSettings, DnsState, ZoneBook};

#[derive(Debug, Clone)]
pub struct DnsView {
    pub settings: Arc<DnsSettings>,
    pub book: Arc<ZoneBook>,
    pub state: DnsState,
}
