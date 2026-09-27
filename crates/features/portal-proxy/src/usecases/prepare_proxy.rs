use portal_feature::{Module, ModulePreparer};
use toml_edit::DocumentMut;

use crate::repositories::{NETWORK, trust_loopback};

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
        trust_loopback(document);
    }
}
