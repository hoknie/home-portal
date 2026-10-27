use portal_feature::{Module, ModulePreparer};
use toml_edit::DocumentMut;

use crate::helpers::{LOOPBACK, covers_loopback};
use crate::repositories::{NETWORK, append_trusted_proxy, trusted_proxies};

pub const TOUCHED: &[&str] = &[NETWORK];

#[derive(Debug, Clone, Copy, Default)]
pub struct PrepareProxy;

impl ModulePreparer for PrepareProxy {
    fn module(&self) -> Module {
        Module::Proxy
    }

    fn touches(&self) -> &'static [&'static str] {
        TOUCHED
    }

    fn prepare(&self, document: &mut DocumentMut) {
        let trusted = trusted_proxies(document);
        if !covers_loopback(trusted.iter().map(String::as_str)) {
            append_trusted_proxy(document, &LOOPBACK.to_string());
        }
    }
}
