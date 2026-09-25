use std::sync::Arc;

use portal_feature::Gate;

use crate::ports::{PublishedServices, TrustedPeers};

#[derive(Clone)]
pub struct ProxyPorts {
    pub services: Arc<dyn PublishedServices>,
    pub gate: Arc<dyn Gate>,
    pub peers: Arc<dyn TrustedPeers>,
}
