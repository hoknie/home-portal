use toml_edit::DocumentMut;

use crate::types::Module;

pub trait ModulePreparer: Send + Sync {
    fn module(&self) -> Module;

    fn touches(&self) -> &'static [&'static str];

    fn prepare(&self, document: &mut DocumentMut);
}
