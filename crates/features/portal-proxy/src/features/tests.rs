use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::Feature;

use super::ProxyFeature;
use crate::fakes::{Catalogue, Loopback, Sessions};
use crate::types::ProxyPorts;

#[test]
fn the_feature_is_named_after_its_crate_and_validates_its_section() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    std::fs::write(&path, "").unwrap();
    let configuration = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = ProxyFeature::new(
        configuration.clone(),
        ProxyPorts {
            services: Arc::new(Catalogue { configuration }),
            gate: Arc::new(Sessions),
            peers: Arc::new(Loopback),
        },
        "127.0.0.1:8080".parse().unwrap(),
    );
    assert_eq!(feature.name(), "proxy");
    assert!(feature.validator().is_some());
    assert_eq!(feature.loops().len(), 1);
}
