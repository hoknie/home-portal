use std::path::PathBuf;

use crate::types::{Refusal, ScriptEntry};

pub trait ScriptLibrary: Send + Sync {
    fn root(&self) -> PathBuf;

    fn resolve(&self, script: &str) -> Result<PathBuf, Refusal>;

    fn list(&self) -> Option<Vec<ScriptEntry>>;

    fn editing(&self) -> bool;
}
