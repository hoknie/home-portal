use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use toml_edit::DocumentMut;

use super::{Origins, Revision};

pub type TypedSections = Arc<Mutex<HashMap<(TypeId, usize), Arc<dyn Any + Send + Sync>>>>;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub document: Arc<DocumentMut>,
    pub revision: Revision,
    pub origins: Arc<Origins>,
    pub sections: TypedSections,
}

impl Snapshot {
    pub fn new(document: DocumentMut, revision: Revision, origins: Origins) -> Snapshot {
        Snapshot {
            document: Arc::new(document),
            revision,
            origins: Arc::new(origins),
            sections: Arc::default(),
        }
    }

    pub fn typed<T, E>(&self, read: fn(&DocumentMut) -> Result<T, E>) -> Result<Arc<T>, E>
    where
        T: Send + Sync + 'static,
    {
        let key = (TypeId::of::<T>(), read as usize);
        let known = self
            .sections
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
            .cloned();
        if let Some(known) = known.and_then(|value| value.downcast::<T>().ok()) {
            return Ok(known);
        }
        let parsed = Arc::new(read(&self.document)?);
        self.sections
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key, parsed.clone());
        Ok(parsed)
    }
}
