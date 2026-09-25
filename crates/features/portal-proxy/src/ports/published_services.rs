use crate::types::PublishedService;

pub trait PublishedServices: Send + Sync {
    fn published(&self) -> Vec<PublishedService>;
}
